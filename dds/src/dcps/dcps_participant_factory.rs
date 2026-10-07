use crate::{
    dcps::{
        dcps_domain_participant::participant_entity::{
            DcpsDomainParticipant, ParticipantSecurityData,
        },
        listeners::domain_participant_listener::DcpsDomainParticipantListener,
        status_mask::StatusMask,
    },
    dds_async::domain_participant_factory::DcpsSender,
    infrastructure::{
        configuration::DustDdsConfiguration,
        domain::DomainId,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        time::{Duration, Time},
    },
    runtime::DdsRuntime,
    security::{
        plugins::{
            access_control::AccessControl, authentication::Authentication,
            cryptographic::Cryptographic, types::DdsSecurityPlugins,
        },
        types::{
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_DISCOVERY_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_KEY_REVISION_ENABLED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_LIVELINESS_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_AXK_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_PSK_PROTECTED,
            PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID,
            PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_ALLOW_UNAUTHENTICATED_PARTICIPANTS,
            PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_IS_ACCESS_PROTECTED,
            ParticipantSecurityAlgorithmInfo, ParticipantSecurityAttributesMaskExt,
            ParticipantSecurityConfig, ParticipantSecurityProtectionInfo,
        },
    },
    transport::{
        interface::RtpsTransportParticipant,
        types::{ENTITYID_PARTICIPANT, Guid, GuidPrefix},
    },
};
use alloc::{string::String, vec::Vec};

pub struct DcpsParticipantFactory {
    pub domain_participant_list: Vec<DcpsDomainParticipant>,
    pub qos: DomainParticipantFactoryQos,
    pub default_participant_qos: DomainParticipantQos,
    pub configuration: DustDdsConfiguration,
    pub dcps_sender: DcpsSender,
}

impl DcpsParticipantFactory {
    pub fn new(configuration: DustDdsConfiguration, dcps_sender: DcpsSender) -> Self {
        Self {
            domain_participant_list: Default::default(),
            qos: Default::default(),
            default_participant_qos: Default::default(),
            configuration,
            dcps_sender,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_participant<Auth, Access, Crypto>(
        &mut self,
        guid_prefix: GuidPrefix,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        dcps_listener: Option<DcpsDomainParticipantListener>,
        listener_mask: StatusMask,
        transport_participant: RtpsTransportParticipant,
        now: Time,
        runtime: &impl DdsRuntime,
        security_plugins: &mut Option<DdsSecurityPlugins<Auth, Access, Crypto>>,
    ) -> DdsResult<InstanceHandle>
    where
        Auth: Authentication,
        Access: AccessControl,
        Crypto: Cryptographic,
    {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let listener_sender = dcps_listener.map(|l| l.spawn(&runtime.spawner()));

        let candidate_participant_guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);
        let (guid, security_data) = if let Some(security) = security_plugins {
            // Step 1: validate_local_identity
            let validate_out = security
                .authentication_plugin
                .validate_local_identity(
                    domain_id,
                    &domain_participant_qos,
                    candidate_participant_guid,
                )
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 2: validate_local_permissions
            let permissions_handle = security
                .access_control_plugin
                .validate_local_permissions(
                    &mut security.authentication_plugin,
                    &validate_out.local_identity_handle,
                    domain_id,
                    &domain_participant_qos,
                )
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 3: check_create_participant
            security
                .access_control_plugin
                .check_create_participant(&permissions_handle, domain_id, &domain_participant_qos)
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 4: get_identity_token
            let identity_token = security
                .authentication_plugin
                .get_identity_token(&validate_out.local_identity_handle)
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 5: get_identity_status_token
            let identity_status_token = security
                .authentication_plugin
                .get_identity_status_token(&validate_out.local_identity_handle)
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 6: get_permissions_token
            let permissions_token = security
                .access_control_plugin
                .get_permissions_token(&permissions_handle)
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 7: get_permissions_credential_token
            let permissions_credential_token = security
                .access_control_plugin
                .get_permissions_credential_token(&permissions_handle)
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 8: set_permissions_credential_and_token
            security
                .authentication_plugin
                .set_permissions_credential_and_token(
                    &validate_out.local_identity_handle,
                    permissions_credential_token,
                )
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 9: get_participant_security_config
            let participant_security_config = security
                .access_control_plugin
                .get_participant_security_config(&permissions_handle)
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 10: set_participant_security_config on Authentication
            let auth_algorithm_info = security
                .authentication_plugin
                .set_participant_security_config(
                    &validate_out.local_identity_handle,
                    &participant_security_config,
                )
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            // Step 11: register_local_participant on Cryptography
            let crypto_out = security
                .cryptographic_plugin
                .register_local_participant(
                    &mut security.authentication_plugin,
                    &mut security.access_control_plugin,
                    &validate_out.local_identity_handle,
                    &permissions_handle,
                    domain_participant_qos
                        .property
                        .value
                        .iter()
                        .filter(|p| p.name.starts_with("dds.sec.crypto.")),
                    &participant_security_config,
                )
                .map_err(|_| DdsError::NotAllowedBySecurity)?;

            let algorithm_info = ParticipantSecurityAlgorithmInfo {
                digital_signature: auth_algorithm_info.digital_signature,
                key_establishment: auth_algorithm_info.key_establishment,
                symmetric_cipher: crypto_out.adjusted_algorithm_info.symmetric_cipher,
            };

            let protection_info = (&participant_security_config).into();

            let security_data = ParticipantSecurityData {
                identity_token,
                permissions_token,
                protection_info,
                algorithm_info,
                identity_status_token,
            };

            // This configure operation is internal to the DDS implementation and therefore this API
            // is not specified by the DDS Security specification. It is mentioned here to provide guidance
            // to implementers. The DomainParticipant’s IdentityToken, the
            // PermissionsToken, the ParticipantSecurityConfig returned by
            // get_participant_security_config and the
            // ParticipantSecurityAlgorithmInfo values returned by the two calls to
            // set_participant_security_config are used to configure DDS discovery and
            // also impact the information propagated inside the ParticipantBuiltinTopicData and
            // ParticipantBuiltinTopicDataSecure:

            (validate_out.adjusted_participant_guid, Some(security_data))
        } else {
            (candidate_participant_guid, None)
        };

        let mut dcps_participant = DcpsDomainParticipant::new(
            domain_id,
            guid,
            domain_participant_qos,
            listener_sender,
            listener_mask,
            transport_participant,
            self.dcps_sender.clone(),
            security_data,
        );
        let participant_handle = *dcps_participant.get_instance_handle();

        if self.qos.entity_factory.autoenable_created_entities {
            dcps_participant
                .enable_domain_participant(now, self.configuration.domain_tag().to_string())?;
        }

        self.domain_participant_list.push(dcps_participant);

        Ok(participant_handle)
    }

    pub fn delete_participant(
        &mut self,
        participant_handle: &InstanceHandle,
        now: Time,
    ) -> DdsResult<()> {
        let index = self
            .domain_participant_list
            .iter()
            .position(|h| h.get_instance_handle() == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)?;
        if !self.domain_participant_list[index].is_participant_empty() {
            return Err(DdsError::PreconditionNotMet(String::from(
                "Domain participant still contains other entities",
            )));
        }
        let mut participant = self.domain_participant_list.remove(index);
        participant.announce_deleted_participant(now);
        Ok(())
    }

    pub fn find_participant(
        &mut self,
        participant_handle: &InstanceHandle,
    ) -> DdsResult<&mut DcpsDomainParticipant> {
        self.domain_participant_list
            .iter_mut()
            .find(|x| x.get_instance_handle() == participant_handle)
            .ok_or(DdsError::AlreadyDeleted)
    }

    pub fn set_default_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    pub fn get_default_participant_qos(&mut self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&mut self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }

    pub fn time_until_next_event(&self, now: Time) -> Option<Duration> {
        self.domain_participant_list
            .iter()
            .filter_map(|x| {
                x.time_until_next_event(
                    now,
                    self.configuration
                        .participant_announcement_interval()
                        .into(),
                )
            })
            .min()
    }
}

impl From<&ParticipantSecurityConfig> for ParticipantSecurityProtectionInfo {
    fn from(config: &ParticipantSecurityConfig) -> Self {
        let mut mask = PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_VALID;
        if config.is_rtps_axk_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_AXK_PROTECTED;
        }
        if config.is_discovery_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_DISCOVERY_PROTECTED;
        }
        if config.is_liveliness_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_LIVELINESS_PROTECTED;
        }
        if config.is_key_revision_enabled {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_KEY_REVISION_ENABLED;
        }
        if config.is_rtps_psk_protected {
            mask |= PARTICIPANT_SECURITY_ATTRIBUTES_FLAG_IS_RTPS_PSK_PROTECTED;
        }

        let mut opt_mask: u16 = 0;
        let mut opt_is_set: u16 = 0;

        opt_is_set |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_ALLOW_UNAUTHENTICATED_PARTICIPANTS;
        if config.allow_unauthenticated_participants {
            opt_mask |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_ALLOW_UNAUTHENTICATED_PARTICIPANTS;
        }

        opt_is_set |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_IS_ACCESS_PROTECTED;
        if config.is_access_protected {
            opt_mask |= PARTICIPANT_SECURITY_OPT_ATTRIBUTES_FLAG_IS_ACCESS_PROTECTED;
        }

        Self {
            participant_security_attributes: mask,
            plugin_participant_security_attributes: config.plugin_participant_attributes,
            participant_security_optional_attributes: ParticipantSecurityAttributesMaskExt {
                is_set: opt_is_set,
                value: opt_mask,
            },
        }
    }
}
