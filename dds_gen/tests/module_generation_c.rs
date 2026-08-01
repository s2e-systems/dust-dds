use std::path::Path;

#[test]
fn module_generation() {
    let idl_file = Path::new("tests/module_generation.idl");
    let expected = r#"
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

    static inline const DustDdsDynamicType* Game_Chess_ChessSquare_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "Game::Chess::ChessSquare",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "column",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_CHAR8),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "line",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_UINT16),
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

    static inline struct Game_Chess_ChessSquare Game_Chess_ChessSquare_create_sample(DustDdsDynamicData* src) {
        struct Game_Chess_ChessSquare sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_char8_value(src, 0, &sample.column);
        dds_dynamic_data_get_uint16_value(src, 1, &sample.line);
        return sample;
    }

    static inline DustDdsDynamicData* Game_Chess_ChessSquare_create_dynamic_sample(const struct Game_Chess_ChessSquare* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(Game_Chess_ChessSquare_get_type());
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

    static inline ReturnCode dds_datawriter_write_Game_Chess_ChessSquare(DustDdsDataWriter* writer, const struct Game_Chess_ChessSquare* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Game_Chess_ChessSquare_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dds_datareader_read_Game_Chess_ChessSquare(DustDdsDataReader* reader, struct Game_Chess_ChessSquare* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, max_samples, received_samples);
        if (result == RETCODE_OK) {
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

    static inline const DustDdsDynamicType* Point_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "Point",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "x",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
                    .is_key = false,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "y",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_FLOAT64),
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

    static inline struct Point Point_create_sample(DustDdsDynamicData* src) {
        struct Point sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_float64_value(src, 0, &sample.x);
        dds_dynamic_data_get_float64_value(src, 1, &sample.y);
        return sample;
    }

    static inline DustDdsDynamicData* Point_create_dynamic_sample(const struct Point* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(Point_get_type());
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

    static inline ReturnCode dds_datawriter_write_Point(DustDdsDataWriter* writer, const struct Point* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = Point_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dds_datareader_read_Point(DustDdsDataReader* reader, struct Point* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, max_samples, received_samples);
        if (result == RETCODE_OK) {
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
    typedef int32_t foo_Bar;
    typedef int32_t foo_Car;
    struct foo_frob_Baz {
        foo_Bar qux;
        foo_Car qix;
    };

    static inline const DustDdsDynamicType* foo_frob_Baz_get_type(void) {
        static const DustDdsDynamicType* type = NULL;
        if (type == NULL) {
            DustDdsTypeDescriptor descriptor = {
                .kind = TYPE_KIND_STRUCTURE,
                .name = "foo::frob::Baz",
                .base_type = NULL,
                .discriminator_type = NULL,
                .bound = NULL,
                .element_type = NULL,
                .key_element_type = NULL,
                .extensibility_kind = EXTENSIBILITY_KIND_FINAL,
                .is_nested = false
            };
            DustDdsDynamicTypeBuilder* builder = dds_dynamic_type_builder_factory_create_type(&descriptor);
            {
                DustDdsMemberDescriptor member = {
                    .name = "qux",
                    .id = 0,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32),
                    .is_key = true,
                    .is_optional = false,
                    .is_must_understand = true
                };
                dds_dynamic_type_builder_add_member(builder, &member);
            }
            {
                DustDdsMemberDescriptor member = {
                    .name = "qix",
                    .id = 1,
                    .type = dds_dynamic_type_get_primitive_type(TYPE_KIND_INT32),
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

    static inline struct foo_frob_Baz foo_frob_Baz_create_sample(DustDdsDynamicData* src) {
        struct foo_frob_Baz sample;
        memset(&sample, 0, sizeof(sample));
        dds_dynamic_data_get_int32_value(src, 0, &sample.qux);
        dds_dynamic_data_get_int32_value(src, 1, &sample.qix);
        return sample;
    }

    static inline DustDdsDynamicData* foo_frob_Baz_create_dynamic_sample(const struct foo_frob_Baz* src) {
        DustDdsDynamicData* sample = dds_dynamic_data_create(foo_frob_Baz_get_type());
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

    static inline ReturnCode dds_datawriter_write_foo_frob_Baz(DustDdsDataWriter* writer, const struct foo_frob_Baz* data) {
        if (writer == NULL || data == NULL) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData* sample = foo_frob_Baz_create_dynamic_sample(data);
        if (sample == NULL) {
            return RETCODE_ERROR;
        }
        ReturnCode result = dds_datawriter_write(writer, sample);
        dds_dynamic_data_free(sample);
        return result;
    }

    static inline ReturnCode dds_datareader_read_foo_frob_Baz(DustDdsDataReader* reader, struct foo_frob_Baz* data_values, int32_t max_samples, int32_t* received_samples) {
        if (reader == NULL || data_values == NULL || received_samples == NULL || max_samples <= 0) {
            return RETCODE_BAD_PARAMETER;
        }
        DustDdsDynamicData** samples = (DustDdsDynamicData**)calloc(max_samples, sizeof(DustDdsDynamicData*));
        if (samples == NULL) {
            return RETCODE_OUT_OF_RESOURCES;
        }
        ReturnCode result = dds_datareader_read(reader, samples, max_samples, received_samples);
        if (result == RETCODE_OK) {
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
"#;

    let result = dust_dds_gen::compile_idl_c(idl_file).unwrap();

    assert_eq!(result, expected);
}
