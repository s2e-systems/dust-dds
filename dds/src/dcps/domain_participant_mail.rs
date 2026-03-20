impl<R: DdsRuntime> DcpsDomainParticipant<R> {
    async fn handle_topic_service(&mut self, topic_service_mail: TopicServiceMail) {
        match topic_service_mail {}
    }

    async fn handle_publisher_service(&mut self, publisher_service_mail: PublisherServiceMail) {
        match publisher_service_mail {
            PublisherServiceMail::CreateDataWriter {
                publisher_handle,
                topic_name,
                qos,
                dcps_listener,
                mask,
                dcps_sender,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.create_data_writer(
                    publisher_handle,
                    topic_name,
                    qos,
                    dcps_listener,
                    mask,
                    dcps_sender,
                    participant_address,
                )
                .await,
            ),
            PublisherServiceMail::DeleteDataWriter {
                publisher_handle,
                datawriter_handle,
                reply_sender,
            } => reply_sender.send(
                self.delete_data_writer(publisher_handle, datawriter_handle)
                    .await,
            ),
            PublisherServiceMail::GetDefaultDataWriterQos {
                publisher_handle,
                reply_sender,
            } => reply_sender.send(self.get_default_datawriter_qos(publisher_handle)),
            PublisherServiceMail::SetDefaultDataWriterQos {
                publisher_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_default_datawriter_qos(publisher_handle, qos)),
            PublisherServiceMail::GetPublisherQos {
                publisher_handle,
                reply_sender,
            } => reply_sender.send(self.get_publisher_qos(publisher_handle)),
            PublisherServiceMail::SetPublisherQos {
                publisher_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_publisher_qos(publisher_handle, qos)),
            PublisherServiceMail::SetPublisherListener {
                publisher_handle,
                dcps_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.set_publisher_listener(
                publisher_handle,
                dcps_listener,
                mask,
            )),
        }
    }

    async fn handle_writer_service(&mut self, writer_service_mail: WriterServiceMail) {
        match writer_service_mail {
            WriterServiceMail::SetListener {
                publisher_handle,
                data_writer_handle,
                dcps_listener,
                listener_mask,
                reply_sender,
            } => reply_sender.send(self.set_listener_data_writer(
                publisher_handle,
                data_writer_handle,
                dcps_listener,
                listener_mask,
            )),
            WriterServiceMail::GetDataWriterQos {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(self.get_data_writer_qos(publisher_handle, data_writer_handle)),
            WriterServiceMail::GetMatchedSubscriptions {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender
                .send(self.get_matched_subscriptions(publisher_handle, data_writer_handle)),
            WriterServiceMail::GetMatchedSubscriptionData {
                publisher_handle,
                data_writer_handle,
                subscription_handle,
                reply_sender,
            } => reply_sender.send(self.get_matched_subscription_data(
                publisher_handle,
                data_writer_handle,
                subscription_handle,
            )),
            WriterServiceMail::GetPublicationMatchedStatus {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(
                self.get_publication_matched_status(publisher_handle, data_writer_handle)
                    .await,
            ),
            WriterServiceMail::UnregisterInstance {
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(
                self.unregister_instance(
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                )
                .await,
            ),
            WriterServiceMail::LookupInstance {
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                reply_sender,
            } => reply_sender.send(self.lookup_instance(
                publisher_handle,
                data_writer_handle,
                dynamic_data,
            )),
            WriterServiceMail::WriteWTimestamp {
                dcps_sender,
                participant_address,
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
                reply_sender,
            } => {
                self.write_w_timestamp(
                    dcps_sender,
                    participant_address,
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                    reply_sender,
                )
                .await
            }

            WriterServiceMail::DisposeWTimestamp {
                publisher_handle,
                data_writer_handle,
                dynamic_data,
                timestamp,
                reply_sender,
            } => reply_sender.send(
                self.dispose_w_timestamp(
                    publisher_handle,
                    data_writer_handle,
                    dynamic_data,
                    timestamp,
                )
                .await,
            ),
            WriterServiceMail::GetOfferedDeadlineMissedStatus {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(
                self.get_offered_deadline_missed_status(publisher_handle, data_writer_handle)
                    .await,
            ),
            WriterServiceMail::EnableDataWriter {
                publisher_handle,
                data_writer_handle,
                dcps_sender,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.enable_data_writer(
                    publisher_handle,
                    data_writer_handle,
                    dcps_sender,
                    participant_address,
                )
                .await,
            ),
            WriterServiceMail::SetDataWriterQos {
                publisher_handle,
                data_writer_handle,
                qos,
                dcps_sender,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.set_data_writer_qos(
                    publisher_handle,
                    data_writer_handle,
                    qos,
                    dcps_sender,
                    participant_address,
                )
                .await,
            ),
        }
    }

    async fn handle_subscriber_service(&mut self, subscriber_service_mail: SubscriberServiceMail) {
        match subscriber_service_mail {
            SubscriberServiceMail::CreateDataReader {
                subscriber_handle,
                topic_name,
                qos,
                dcps_listener,
                mask,
                dcps_sender,
                domain_participant_address,
                reply_sender,
            } => reply_sender.send(
                self.create_data_reader(
                    subscriber_handle,
                    topic_name,
                    qos,
                    dcps_listener,
                    mask,
                    dcps_sender,
                    domain_participant_address,
                )
                .await,
            ),
            SubscriberServiceMail::DeleteDataReader {
                subscriber_handle,
                datareader_handle,
                reply_sender,
            } => reply_sender.send(
                self.delete_data_reader(subscriber_handle, datareader_handle)
                    .await,
            ),
            SubscriberServiceMail::LookupDataReader {
                subscriber_handle,
                topic_name,
                reply_sender,
            } => reply_sender.send(self.lookup_data_reader(subscriber_handle, topic_name)),
            SubscriberServiceMail::SetDefaultDataReaderQos {
                subscriber_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_default_data_reader_qos(subscriber_handle, qos)),
            SubscriberServiceMail::GetDefaultDataReaderQos {
                subscriber_handle,
                reply_sender,
            } => reply_sender.send(self.get_default_data_reader_qos(subscriber_handle)),
            SubscriberServiceMail::SetQos {
                subscriber_handle,
                qos,
                reply_sender,
            } => reply_sender.send(self.set_subscriber_qos(subscriber_handle, qos)),
            SubscriberServiceMail::GetSubscriberQos {
                subscriber_handle,
                reply_sender,
            } => reply_sender.send(self.get_subscriber_qos(subscriber_handle)),
            SubscriberServiceMail::SetListener {
                subscriber_handle,
                dcps_listener,
                mask,
                reply_sender,
            } => reply_sender.send(self.set_subscriber_listener(
                subscriber_handle,
                dcps_listener,
                mask,
            )),
        }
    }

    async fn handle_reader_service(&mut self, reader_service_mail: ReaderServiceMail) {
        match reader_service_mail {
            ReaderServiceMail::Read {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
                reply_sender,
            } => reply_sender.send(
                self.read(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    specific_instance_handle,
                )
                .await,
            ),
            ReaderServiceMail::Take {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                sample_states,
                view_states,
                instance_states,
                specific_instance_handle,
                reply_sender,
            } => reply_sender.send(
                self.take(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    specific_instance_handle,
                )
                .await,
            ),
            ReaderServiceMail::ReadNextInstance {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
                reply_sender,
            } => reply_sender.send(
                self.read_next_instance(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    previous_handle,
                    sample_states,
                    view_states,
                    instance_states,
                )
                .await,
            ),
            ReaderServiceMail::TakeNextInstance {
                subscriber_handle,
                data_reader_handle,
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
                reply_sender,
            } => reply_sender.send(
                self.take_next_instance(
                    subscriber_handle,
                    data_reader_handle,
                    max_samples,
                    previous_handle,
                    sample_states,
                    view_states,
                    instance_states,
                )
                .await,
            ),
            ReaderServiceMail::Enable {
                subscriber_handle,
                data_reader_handle,
                dcps_sender,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.enable_data_reader(
                    subscriber_handle,
                    data_reader_handle,
                    dcps_sender,
                    participant_address,
                )
                .await,
            ),
            ReaderServiceMail::GetSubscriptionMatchedStatus {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender.send(
                self.get_subscription_matched_status(subscriber_handle, data_reader_handle)
                    .await,
            ),
            ReaderServiceMail::WaitForHistoricalData {
                participant_address,
                subscriber_handle,
                data_reader_handle,
                max_wait,
                reply_sender,
            } => reply_sender.send(self.wait_for_historical_data(
                participant_address,
                subscriber_handle,
                data_reader_handle,
                max_wait,
            )),
            ReaderServiceMail::GetMatchedPublicationData {
                subscriber_handle,
                data_reader_handle,
                publication_handle,
                reply_sender,
            } => reply_sender.send(self.get_matched_publication_data(
                subscriber_handle,
                data_reader_handle,
                publication_handle,
            )),
            ReaderServiceMail::GetMatchedPublications {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender
                .send(self.get_matched_publications(subscriber_handle, data_reader_handle)),
            ReaderServiceMail::GetQos {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender.send(self.get_data_reader_qos(subscriber_handle, data_reader_handle)),
            ReaderServiceMail::SetQos {
                subscriber_handle,
                data_reader_handle,
                qos,
                dcps_sender,
                participant_address,
                reply_sender,
            } => reply_sender.send(
                self.set_data_reader_qos(
                    subscriber_handle,
                    data_reader_handle,
                    qos,
                    dcps_sender,
                    participant_address,
                )
                .await,
            ),
            ReaderServiceMail::SetListener {
                subscriber_handle,
                data_reader_handle,
                dcps_listener,
                listener_mask,
                reply_sender,
            } => reply_sender.send(self.set_data_reader_listener(
                subscriber_handle,
                data_reader_handle,
                dcps_listener,
                listener_mask,
            )),
        }
    }

    async fn handle_message_service(&mut self, message_service_mail: MessageServiceMail) {
        match message_service_mail {
            MessageServiceMail::AddCacheChange {
                dcps_sender,
                cache_change,
                subscriber_handle,
                data_reader_handle,
            } => {
                self.add_cache_change(
                    dcps_sender,
                    cache_change,
                    subscriber_handle,
                    data_reader_handle,
                )
                .await
            }
            MessageServiceMail::RemoveWriterChange {
                publisher_handle,
                data_writer_handle,
                sequence_number,
            } => {
                self.remove_writer_change(publisher_handle, data_writer_handle, sequence_number)
                    .await
            }
            MessageServiceMail::AreAllChangesAcknowledged {
                publisher_handle,
                data_writer_handle,
                reply_sender,
            } => reply_sender.send(
                self.are_all_changes_acknowledged(publisher_handle, data_writer_handle)
                    .await,
            ),
            MessageServiceMail::IsHistoricalDataReceived {
                subscriber_handle,
                data_reader_handle,
                reply_sender,
            } => reply_sender.send(
                self.is_historical_data_received(subscriber_handle, data_reader_handle)
                    .await,
            ),
            MessageServiceMail::AddBuiltinParticipantsDetectorCacheChange {
                cache_change,
                dcps_sender,
                participant_address,
            } => {
                self.add_builtin_participants_detector_cache_change(cache_change, dcps_sender)
                    .await
            }
            MessageServiceMail::AddBuiltinPublicationsDetectorCacheChange {
                cache_change,
                dcps_sender,
                participant_address,
            } => {
                self.add_builtin_publications_detector_cache_change(cache_change, dcps_sender)
                    .await;
            }
            MessageServiceMail::AddBuiltinSubscriptionsDetectorCacheChange {
                cache_change,
                dcps_sender,
                participant_address,
            } => {
                self.add_builtin_subscriptions_detector_cache_change(cache_change, dcps_sender)
                    .await
            }
            MessageServiceMail::AddBuiltinTopicsDetectorCacheChange { cache_change } => {
                self.add_builtin_topics_detector_cache_change(cache_change)
                    .await
            }
            MessageServiceMail::Poke => self.poke().await,
        }
    }

    async fn handle_event_service(&mut self, event_service_mail: EventServiceMail) {
        match event_service_mail {
            EventServiceMail::OfferedDeadlineMissed {
                publisher_handle,
                data_writer_handle,
                change_instance_handle,
                dcps_sender,
                participant_address,
            } => {
                self.offered_deadline_missed(
                    publisher_handle,
                    data_writer_handle,
                    change_instance_handle,
                    dcps_sender,
                    participant_address,
                )
                .await
            }
            EventServiceMail::RequestedDeadlineMissed {
                subscriber_handle,
                data_reader_handle,
                change_instance_handle,
                dcps_sender,
                participant_address,
            } => {
                self.requested_deadline_missed(
                    subscriber_handle,
                    data_reader_handle,
                    change_instance_handle,
                    dcps_sender,
                    participant_address,
                )
                .await
            }
        }
    }

    async fn handle_discovery_service(&mut self, discovery_service_mail: DiscoveryServiceMail) {
        match discovery_service_mail {
            DiscoveryServiceMail::AnnounceParticipant {
                dcps_sender,
                participant_address,
            } => {
                self.announce_participant(dcps_sender, participant_address)
                    .await;
            }
            DiscoveryServiceMail::AnnounceDeletedParticipant => {
                self.announce_deleted_participant().await;
            }
        }
    }
}
