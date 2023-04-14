# Dust DDS Configuration

*The environment DUST_DDS_CONFIGURATION variable can be set as a json to modify the default configuration. E.g. $Env:DUST_DDS_CONFIGURATION='{"interface_name":"Wi-Fi"}'*

## Properties

- **`domain_tag`** *(string)*: Domain tag to use for the participant. Default: ``.
- **`fragment_size`** *(integer)*: Data is fragmented into max size of this. Minimum: `8.0`. Default: `1344`.
- **`interface_name`** *(['string', 'null'])*: Network interface name to use for discovery. Default: `None`.
