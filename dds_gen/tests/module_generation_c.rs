use std::path::Path;

#[test]
fn module_generation() {
    let idl_file = Path::new("tests/module_generation.idl");
    let expected = r###"
    #include <stdbool.h>
    #include <stdint.h>
    #include <stddef.h>
    #include <stdlib.h>
    #include <string.h>
    #include "dust_dds.h"

    enum Game_Chess_ChessPiece {
        Game_Chess_ChessPiece_Pawn,
        Game_Chess_ChessPiece_Rook,
        Game_Chess_ChessPiece_Knight,
        Game_Chess_ChessPiece_Bishop,
        Game_Chess_ChessPiece_Queen,
        Game_Chess_ChessPiece_King
    };
    struct Game_Chess_ChessSquare {
        char column;
        uint16_t line;
    };

    static inline const DDS_DustDdsDynamicType* Game_Chess_ChessSquare_get_type(void) {
        static const DDS_DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DDS_DustDdsTypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "Game::Chess::ChessSquare",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DDS_DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "column",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(DDS_TYPE_KIND_CHAR8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "line",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(DDS_TYPE_KIND_UINT16),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            type = dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct Game_Chess_ChessSquare Game_Chess_ChessSquare_create_sample(DDS_DustDdsDynamicData* src) {
        struct Game_Chess_ChessSquare sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_char8_value(src, 0, &sample.column);
        dds_dynamic_data_get_uint16_value(src, 1, &sample.line);
        return sample;
    }

    static inline DDS_DustDdsDynamicData* Game_Chess_ChessSquare_create_dynamic_sample(const struct Game_Chess_ChessSquare* src) {
        DDS_DustDdsDynamicData* sample = dds_dynamic_data_create(Game_Chess_ChessSquare_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_char8_value(sample, 0, src->column);
            dds_dynamic_data_set_uint16_value(sample, 1, src->line);
        }
        return sample;
    }

    static inline void Game_Chess_ChessSquare_free_sample(struct Game_Chess_ChessSquare* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_write(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_write_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_register_instance(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_register_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_unregister_instance(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_unregister_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_dispose(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_dispose_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_get_key_value(DDS_DustDdsDataWriter* writer, struct Game_Chess_ChessSquare* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Game_Chess_ChessSquare_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datawriter_lookup_instance(DDS_DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_read(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Game_Chess_ChessSquare_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_take(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Game_Chess_ChessSquare_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_read_next_sample(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Game_Chess_ChessSquare_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_take_next_sample(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Game_Chess_ChessSquare_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_read_instance(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Game_Chess_ChessSquare_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_take_instance(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Game_Chess_ChessSquare_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_read_next_instance(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Game_Chess_ChessSquare_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_take_next_instance(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Game_Chess_ChessSquare_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_return_loan(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, struct DDS_SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_get_key_value(DDS_DustDdsDataReader* reader, struct Game_Chess_ChessSquare* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Game_Chess_ChessSquare_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Game_Chess_ChessSquare_dds_datareader_lookup_instance(DDS_DustDdsDataReader* reader, const struct Game_Chess_ChessSquare* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datareader_lookup_instance(reader, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }
    enum Game_Cards_Suit {
        Game_Cards_Suit_Spades,
        Game_Cards_Suit_Hearts,
        Game_Cards_Suit_Diamonds,
        Game_Cards_Suit_Clubs
    };
    struct Point {
        double x;
        double y;
    };

    static inline const DDS_DustDdsDynamicType* Point_get_type(void) {
        static const DDS_DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DDS_DustDdsTypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "Point",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DDS_DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "y",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(DDS_TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            type = dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct Point Point_create_sample(DDS_DustDdsDynamicData* src) {
        struct Point sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_float64_value(src, 0, &sample.x);
        dds_dynamic_data_get_float64_value(src, 1, &sample.y);
        return sample;
    }

    static inline DDS_DustDdsDynamicData* Point_create_dynamic_sample(const struct Point* src) {
        DDS_DustDdsDynamicData* sample = dds_dynamic_data_create(Point_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_float64_value(sample, 0, src->x);
            dds_dynamic_data_set_float64_value(sample, 1, src->y);
        }
        return sample;
    }

    static inline void Point_free_sample(struct Point* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode Point_dds_datawriter_write(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_write_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_register_instance(DDS_DustDdsDataWriter* writer, const struct Point* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_register_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_unregister_instance(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_unregister_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_dispose(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_dispose_w_timestamp(DDS_DustDdsDataWriter* writer, const struct Point* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_get_key_value(DDS_DustDdsDataWriter* writer, struct Point* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datawriter_lookup_instance(DDS_DustDdsDataWriter* writer, const struct Point* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read_next_sample(DDS_DustDdsDataReader* reader, struct Point* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take_next_sample(DDS_DustDdsDataReader* reader, struct Point* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = Point_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_read_next_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_take_next_instance(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = Point_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_return_loan(DDS_DustDdsDataReader* reader, struct Point* data_values, struct DDS_SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode Point_dds_datareader_get_key_value(DDS_DustDdsDataReader* reader, struct Point* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = Point_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode Point_dds_datareader_lookup_instance(DDS_DustDdsDataReader* reader, const struct Point* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = Point_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datareader_lookup_instance(reader, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }
    typedef int32_t foo_Bar;
    typedef int32_t foo_Car;
    struct foo_frob_Baz {
        foo_Bar qux;
        foo_Car qix;
    };

    static inline const DDS_DustDdsDynamicType* foo_frob_Baz_get_type(void) {
        static const DDS_DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DDS_DustDdsTypeDescriptor descriptor = {
                .kind = DDS_TYPE_KIND_STRUCTURE,
                .name = "foo::frob::Baz",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = DDS_EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DDS_DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "qux",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(DDS_TYPE_KIND_INT32),
                    .is_key = true,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DDS_DustDdsMemberDescriptor member = {
                    .name = "qix",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(DDS_TYPE_KIND_INT32),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            type = dds_dynamic_type_builder_build(builder);
        }
        return type;
    }

    static inline struct foo_frob_Baz foo_frob_Baz_create_sample(DDS_DustDdsDynamicData* src) {
        struct foo_frob_Baz sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_int32_value(src, 0, &sample.qux);
        dds_dynamic_data_get_int32_value(src, 1, &sample.qix);
        return sample;
    }

    static inline DDS_DustDdsDynamicData* foo_frob_Baz_create_dynamic_sample(const struct foo_frob_Baz* src) {
        DDS_DustDdsDynamicData* sample = dds_dynamic_data_create(foo_frob_Baz_get_type());
        if (sample != NULL) {
            dds_dynamic_data_set_int32_value(sample, 0, src->qux);
            dds_dynamic_data_set_int32_value(sample, 1, src->qix);
        }
        return sample;
    }

    static inline void foo_frob_Baz_free_sample(struct foo_frob_Baz* sample) {
        if (sample != NULL) {
        }
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_write(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_write(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_write_w_timestamp(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_write_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_register_instance(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_register_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_register_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, struct DDS_Time_t source_timestamp, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_register_instance_w_timestamp(writer, sample, source_timestamp, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_unregister_instance(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_unregister_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_unregister_instance_w_timestamp(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_unregister_instance_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_dispose(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_dispose(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_dispose_w_timestamp(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* data, const DDS_InstanceHandle_t* handle, struct DDS_Time_t source_timestamp) {
        if (writer == NULL || data == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_dispose_w_timestamp(writer, sample, handle, source_timestamp);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_get_key_value(DDS_DustDdsDataWriter* writer, struct foo_frob_Baz* key_holder, const DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_get_key_value(writer, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = foo_frob_Baz_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datawriter_lookup_instance(DDS_DustDdsDataWriter* writer, const struct foo_frob_Baz* key_holder, DDS_InstanceHandle_t* handle) {
        if (writer == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datawriter_lookup_instance(writer, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_read(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = foo_frob_Baz_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_take(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take(reader, samples, sample_infos, max_samples, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = foo_frob_Baz_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_read_next_sample(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = dds_datareader_read_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = foo_frob_Baz_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_take_next_sample(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_value, struct DDS_SampleInfo* sample_info) {
        if (reader == NULL || data_value == NULL || sample_info == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = NULL;
        DDS_ReturnCode result = dds_datareader_take_next_sample(reader, &sample, sample_info);
        if (result == DDS_RETCODE_OK) {
            if (sample != NULL) {
                *data_value = foo_frob_Baz_create_sample(sample);
                dds_dynamic_data_free(sample);
            }
        }
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_read_instance(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = foo_frob_Baz_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_take_instance(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* a_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || a_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take_instance(reader, samples, sample_infos, max_samples, a_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = foo_frob_Baz_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_read_next_instance(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_read_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = foo_frob_Baz_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_take_next_instance(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos, int32_t max_samples, const DDS_InstanceHandle_t* previous_handle, DDS_SampleStateMask sample_states, DDS_ViewStateMask view_states, DDS_InstanceStateMask instance_states, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || previous_handle == NULL || received_samples == NULL || max_samples <= 0) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData** samples = (DDS_DustDdsDynamicData**)calloc(max_samples, sizeof(DDS_DustDdsDynamicData*));
        if (samples == NULL) {
            return DDS_RETCODE_OUT_OF_RESOURCES;
        }
        DDS_ReturnCode result = dds_datareader_take_next_instance(reader, samples, sample_infos, max_samples, previous_handle, sample_states, view_states, instance_states, received_samples);
        if (result == DDS_RETCODE_OK) {
            for (int32_t i = 0; i < *received_samples; i++) {
                if (samples[i] != NULL) {
                    data_values[i] = foo_frob_Baz_create_sample(samples[i]);
                    dds_dynamic_data_free(samples[i]);
                }
            }
        }
        free(samples);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_return_loan(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* data_values, struct DDS_SampleInfo* sample_infos) {
        return dds_datareader_return_loan(reader, NULL, sample_infos);
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_get_key_value(DDS_DustDdsDataReader* reader, struct foo_frob_Baz* key_holder, const DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datareader_get_key_value(reader, sample, handle);
        if (result == DDS_RETCODE_OK) {
            *key_holder = foo_frob_Baz_create_sample(sample);
        }
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline DDS_ReturnCode foo_frob_Baz_dds_datareader_lookup_instance(DDS_DustDdsDataReader* reader, const struct foo_frob_Baz* key_holder, DDS_InstanceHandle_t* handle) {
        if (reader == NULL || key_holder == NULL || handle == NULL) {
            return DDS_RETCODE_BAD_PARAMETER;
        }
        DDS_DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(key_holder);
        if (sample == NULL) {
            return DDS_RETCODE_ERROR;
        }
        DDS_ReturnCode result = dds_datareader_lookup_instance(reader, sample, handle);
        dds_dynamic_data_free(sample);
        return result;
    }
"###;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
