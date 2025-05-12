//
// Created by dheide on 26.03.25.
//
#include <inttypes.h>
#include <gtest/gtest.h>

#include "bitis_lib.h"


namespace test_serializer {
    void print_u8vec_as_hex(std::vector<uint8_t> &vec) {
        for (size_t i = 0; i < vec.size(); i++) {
            printf("%02X ", vec[i]);
        }
    }

    TEST(BITIS_Serialization_BaseTypes, IntWithSize) {
        auto ser = BitisSerializer();

        auto data = IntgralWithGivenBitSize<uint8_t, 8>(67);
        data.serialize(ser);
        printf("data: "); data.print(-1); printf("\n");
        auto r = ser.finalize();
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = IntgralWithGivenBitSize<uint8_t, 8>::deserialize(des);
        printf("deserialized data: %d\n", dd.data.value);

        // assert(data.is_equal(dd.data));
        EXPECT_TRUE(data == dd.data);
    }
    TEST(BITIS_Serialization_BaseTypes, DynInt) {
        const auto vals = std::vector<uint8_t>{2, 7, 10};
        for(auto &v : vals) {
            printf("* testing DynInt with value %d\n", v);

            auto ser = BitisSerializer();

            auto data = DynInteger<uint8_t, 3>(v);
            printf("data: "); data.print(-1); printf("\n");
            data.serialize(ser);
            auto r = ser.finalize();
            printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);
            printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");

            // ***
            auto des = BitisDeserializer(ser.data_cache);
            auto dd = DynInteger<uint8_t, 3>::deserialize(des);
            printf("deserialized data: %d\n", dd.data.value);

            // assert(data.is_equal(dd.data));
            EXPECT_TRUE(data == dd.data);
        }
    }
    TEST(BITIS_Serialization_BaseTypes, FixedPrecision_ok) {
        auto vals = std::vector<double>{1.+1./13., 0., 1., 2., 3.};
        for(auto &v : vals) {
            printf("* Testing FixedPrecType, val=%f\n", v);

            auto ser = BitisSerializer();

            auto data = FixPrecisionMinMax<4, 1, 2>(v);
            printf("data: "); data.print(-1); printf("\n");

            data.serialize(ser);
            auto r = ser.finalize();
            printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);
            printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");

            // ***
            auto des = BitisDeserializer(ser.data_cache);
            auto dd = FixPrecisionMinMax<4, 1, 2>::deserialize(des);
            printf("deserialized data: "); dd.data.print(-1); printf("\n");

            EXPECT_EQ(data, dd.data);
            printf("* done\n\n");
        }
    }
    TEST(BITIS_Serialization_BaseTypes, TwoFixedPrecision_ok) {
        printf("* Testing two values\n");

        auto ser = BitisSerializer();

        auto data1 = FixPrecisionMinMax<3, 1, 2>(1.);
        printf("data1: "); data1.print(-1); printf("\n");
        auto data2 = FixPrecisionMinMax<3, 1, 2>(2.);
        printf("data2: "); data2.print(-1); printf("\n");

        data1.serialize(ser);
        data2.serialize(ser);
        auto r = ser.finalize();
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd1 = FixPrecisionMinMax<3, 1, 2>::deserialize(des);
        printf("deserialized data1: "); dd1.data.print(-1); printf("\n");
        auto dd2 = FixPrecisionMinMax<3, 1, 2>::deserialize(des);
        printf("deserialized data2: "); dd2.data.print(-1); printf("\n");

        EXPECT_EQ(data1, dd1.data);
        EXPECT_EQ(data2, dd2.data);
        printf("* done\n\n");
    }
    TEST(BITIS_Serialization_BaseTypes, FixedPrecision_rounding) {
        printf("* Testing two values\n");

        auto ser = BitisSerializer();

        // step in FP is 0.2 so 1.05 -> 1, 1.2 -> 1.2, 1.25 -> 1.3
        auto data1 = FixPrecisionMinMax<3, 1, 2>(1.05);
        printf("data1: "); data1.print(-1); printf("\n");
        auto data2 = FixPrecisionMinMax<3, 1, 2>(1.2);
        printf("data2: "); data2.print(-1); printf("\n");
        auto data3 = FixPrecisionMinMax<3, 1, 2>(1.31);
        printf("data3: "); data3.print(-1); printf("\n");

        data1.serialize(ser);
        data2.serialize(ser);
        data3.serialize(ser);
        auto r = ser.finalize();
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd1 = FixPrecisionMinMax<3, 1, 2>::deserialize(des);
        printf("deserialized data1: "); dd1.data.print(-1); printf("\n");
        auto dd2 = FixPrecisionMinMax<3, 1, 2>::deserialize(des);
        printf("deserialized data2: "); dd2.data.print(-1); printf("\n");
        auto dd3 = FixPrecisionMinMax<3, 1, 2>::deserialize(des);
        printf("deserialized data3: "); dd3.data.print(-1); printf("\n");

        data1.set(1.);
        data3.set(1.4);
        EXPECT_EQ(data1, dd1.data);
        EXPECT_EQ(data2, dd2.data);
        EXPECT_EQ(data3, dd3.data);
        printf("* done\n\n");
    }
    TEST(BITIS_Serialization_BaseTypes, FixedArray) {
        printf("* Testing fixedArray\n");

        auto ser = BitisSerializer();

        // step in FP is 0.2 so 1.05 -> 1, 1.2 -> 1.2, 1.25 -> 1.3
        typedef IntgralWithGivenBitSize<int8_t, 6> ValType;
        auto data = FixedArray<ValType, 5>({ValType(1), ValType(2), ValType(-7), ValType(11), ValType(13)});
        // auto data = FixedArray<ValType, 5>({ValType(63), ValType(63), ValType(63), ValType(63), ValType(63)});
        // auto data = FixedArray<ValType, 5>({ValType(7), ValType(7), ValType(7), ValType(7), ValType(7)});
        printf("org data: "); data.print(-1); printf("\n");
        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = FixedArray<ValType, 5>::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }
    TEST(BITIS_Serialization_BaseTypes, DynArray) {
        printf("* Testing fixedArray\n");

        auto ser = BitisSerializer();
        typedef BitisFloatingPoint<double> ValType;
        auto data = DynArray<ValType, 4>({ValType(1.2), ValType(11111.22222),
            ValType(11111.22222), ValType(-1.)});
        printf("org data: "); data.print(-1); printf("\n");

        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = DynArray<ValType, 4>::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }

    namespace AssetStatesEnum {
        ENUM_INSTANCE(On);
        ENUM_INSTANCE(Off);
        ENUM_INSTANCE(SomewhereInBetween);
        ENUM_INSTANCE(la);
    }
    TEST(BITIS_Serialization_BaseTypes, Enums) {
        printf("* Testing enums\n");

        auto ser = BitisSerializer();

        typedef BitisEnum<bitis_helper::Collector<AssetStatesEnum::Off, AssetStatesEnum::On, AssetStatesEnum::SomewhereInBetween>, 4> ValType;
        auto data = ValType::create_enum<AssetStatesEnum::On>();
        printf("org data: "); data.print(-1); printf("\n");

        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = ValType::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }
    TEST(BITIS_Serialization_BaseTypes, ArrayEnums) {
        printf("* Testing enums\n");

        auto ser = BitisSerializer();

        typedef BitisEnum<bitis_helper::Collector<
            AssetStatesEnum::Off, AssetStatesEnum::On,
            AssetStatesEnum::SomewhereInBetween>, 3> ValType;
        auto data = FixedArray<ValType, 2>({ValType::create_enum<AssetStatesEnum::On>(), ValType::create_enum<AssetStatesEnum::Off>()});
        printf("org data: "); data.print(-1); printf("\n");

        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = FixedArray<ValType, 2>::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }
    TEST(BITIS_Serialization_BaseTypes, DynArrayEnums) {
        printf("* Testing enums\n");

        auto ser = BitisSerializer();

        typedef BitisEnum<bitis_helper::Collector<
            AssetStatesEnum::Off, AssetStatesEnum::On,
            AssetStatesEnum::SomewhereInBetween>, 3> ValType;
        auto data = DynArray<ValType, 4>({ValType::create_enum<AssetStatesEnum::On>(), ValType::create_enum<AssetStatesEnum::Off>()});
        printf("org data: "); data.print(-1); printf("\n");

        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = DynArray<ValType, 4>::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }
    TEST(BITIS_Serialization_BaseTypes, OptionalWithVal) {
        printf("* Testing optional value\n");

        auto ser = BitisSerializer();
        typedef IntgralWithGivenBitSize<int8_t, 5> ValType;
        auto dval = ValType(-4);
        auto data = BitisOptional<ValType>::create_val(dval);

        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = BitisOptional<ValType>::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }
    TEST(BITIS_Serialization_BaseTypes, OptionalNone) {
        printf("* Testing optional value\n");

        auto ser = BitisSerializer();
        typedef IntgralWithGivenBitSize<int8_t, 5> ValType;
        auto data = BitisOptional<ValType>::create_none();

        data.serialize(ser);
        auto r = ser.finalize();
        printf("serialized data: "); print_u8vec_as_hex(ser.data_cache); printf("\n");
        printf("bits: %zu, bytes: %zu\n", r.bits, r.bytes);

        // ***
        auto des = BitisDeserializer(ser.data_cache);
        auto dd = BitisOptional<ValType>::deserialize(des);
        printf("deserialized data: "); dd.data.print(-1); printf("\n");
        EXPECT_TRUE(data==dd.data);
    }
}

namespace test_msg {

    struct MsgA {
        static const char *msg_attr[];

        typedef message_helper::MessageT<
            IntgralWithGivenBitSize<uint16_t, 12>,
            BitisFloatingPoint<float>,
            IntgralWithGivenBitSize<uint8_t, 4>
        > MsgT;

        IntgralWithGivenBitSize<uint16_t, 12> a;
        BitisFloatingPoint<float> b;
        IntgralWithGivenBitSize<uint8_t, 4> c;

        std::size_t serialize(BitisSerializer &ser) const {
            return message_helper::msg_serialize(this, ser);
        }
        static bitis_helper::BitiaDeserializerHelper<MsgA> deserialize(BitisDeserializer &des) {
            return message_helper::msg_deserialize<MsgA>(des);
        }

        void print(int16_t indent=0) {
            printf("MsgA{ ");
            if (indent>=0) printf("\n");
            message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
            print_indent(indent); printf("}");
            // if (indent>=0) printf("\n");
        }
    };
    const char *MsgA::msg_attr[] = {"a", "b", "c"};

    struct MsgB {
        static const char *msg_attr[];

        typedef message_helper::MessageT<
            MsgA, IntgralWithGivenBitSize<uint8_t, 4>
        > MsgT;
        MsgA a;
        IntgralWithGivenBitSize<uint8_t, 4> b;

        std::size_t serialize(BitisSerializer &ser) const {
            return message_helper::msg_serialize(this, ser);
        }
        void print(const int16_t indent=0) {
            printf("MsgB{ ");
            if (indent>=0) printf("\n");
            message_helper::msg_print(this, (indent>=0) ? indent + 2 : indent, msg_attr);
            print_indent(indent); printf("}");
            // if (indent>=0) printf("\n");
        }
    };
    const char *MsgB::msg_attr[] = {"a", "b"};

    TEST(BITIS_Messages, message_a) {
        using namespace message_helper;

        // auto b = MessageT<EnumeratedList<uint8_t, uint16_t, float, uint8_t, uint8_t>::type>();
        // A *a = (A *) &b;
        auto d = MsgA{
            .a = IntgralWithGivenBitSize<uint16_t, 12>(1111),
            .b = BitisFloatingPoint<float>(12.345),
            .c = IntgralWithGivenBitSize<uint8_t, 4>(9)
        };
        auto *a = (MsgA *) &d;
        auto *b = (MsgA::MsgT*) &d;

        printf("a offset %" PRIu64 "\n", (uint64_t)&(a->a) - (uint64_t)&d);
        printf("b offset %" PRIu64 "\n", (uint64_t)&(a->b) - (uint64_t)&d);
        printf("c offset %" PRIu64 "\n", (uint64_t)&(a->c) - (uint64_t)&d);

        printf("attr1 offset %" PRIu64 "\n",
            (uint64_t)(static_cast<MessageAttribute<ETN<IntgralWithGivenBitSize<uint16_t, 12>, 0>>*>(b)) - (uint64_t)&d);
        printf("attr2 offset %" PRIu64 "\n",
            (uint64_t)(static_cast<MessageAttribute<ETN<BitisFloatingPoint<float>, 1>>*>(b)) - (uint64_t)&d);
        printf("attr3 offset %" PRIu64 "\n",
            (uint64_t)(static_cast<MessageAttribute<ETN<IntgralWithGivenBitSize<uint8_t, 4>, 2>>*>(b)) - (uint64_t)&d);

        d.print(0);
        printf("\n");

        //
        auto ser = BitisSerializer();
        // auto r = serialize(a, ser);
        auto r = d.serialize(ser);
        printf("r: %" PRIu64 "\n", r);

        //
    }

    TEST(BITIS_Messages, message_b) {
        // auto b = MessageT<EnumeratedList<uint8_t, uint16_t, float, uint8_t, uint8_t>::type>();
        // A *a = (A *) &b;
        auto d = MsgB {
            .a = MsgA {
                .a = IntgralWithGivenBitSize<uint16_t, 12>(1111),
                .b = BitisFloatingPoint<float>(12.345),
                .c = IntgralWithGivenBitSize<uint8_t, 4>(9)
            },
            .b = IntgralWithGivenBitSize<uint8_t, 4>(5)
        };

        d.print(0);
        printf("\n");

        auto ser = BitisSerializer();
        // auto r = serialize(a, ser);
        auto r = d.serialize(ser);
        printf("r: %" PRIu64 "\n", r);
    }
}


namespace test_oneof {
    struct MsgInner {
        static const char *msg_attr[];
        typedef message_helper::MessageT<
            IntgralWithGivenBitSize<uint16_t, 12>
        > MsgT;

        IntgralWithGivenBitSize<uint16_t, 12> a;

        std::size_t serialize(BitisSerializer &ser) const {
            return message_helper::msg_serialize(this, ser);
        }
        void print(const int16_t indent=0) {
            printf("MsgInner{ ");
            if (indent>=0) printf("\n");
            message_helper::msg_print(this, (indent>=0) ? indent + 2 : indent, msg_attr);
            print_indent(indent); printf("}");
            // if (indent>=0) printf("\n");
        }
    };
    const char *MsgInner::msg_attr[] = {"a"};

    namespace MsgC_helper {
        struct OOEnum_Val {
            static const char *oo_enums[];
            enum OOEnum {
                Inner, FloatVal, SmallInt
            };
            typedef std::integral_constant<uint8_t, 4> SelectorBits;
            typedef oneof_helper::OneOfT<
                MsgInner, BitisFloatingPoint<float>, IntgralWithGivenBitSize<uint8_t, 4>
            > OneOfT;
            OOEnum oo_selector;
            union {
                char _base;
                MsgInner inner;
                BitisFloatingPoint<float> float_val;
                IntgralWithGivenBitSize<uint8_t, 4> small_int;
            } oo;

            explicit OOEnum_Val(const OOEnum _oo_selector) : oo_selector(_oo_selector), oo{(0)} {}

            void set_inner(const MsgInner inner) {
                oo.inner = inner;
                oo_selector = Inner;
            }
            void set_float_val(const BitisFloatingPoint<float> float_val) {
                oo.float_val = float_val;
                oo_selector = FloatVal;
            }
            void set_small_int(const IntgralWithGivenBitSize<uint8_t, 4> small_int) {
                oo.small_int = small_int;
                oo_selector = SmallInt;
            }
            MsgInner *get_inner() {
                if (oo_selector == Inner) { return &oo.inner; }
                return nullptr;
            }
            BitisFloatingPoint<float> *get_float_val() {
                if (oo_selector == FloatVal) { return &oo.float_val; }
                return nullptr;
            }
            IntgralWithGivenBitSize<uint8_t, 4> *get_small_int() {
                if (oo_selector == SmallInt) { return &oo.small_int; }
                return nullptr;
            }

            std::size_t serialize(BitisSerializer &ser) const {
                return oneof_helper::oneof_serialize(this, ser);
            }
            void print(int16_t indent=0) {
                printf("Oneof ");
                oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent, oo_enums);
            }
        };
        const char *OOEnum_Val::oo_enums[] = {"inner", "float_val", "small_int"};

        OOEnum_Val OOEnum_Val_Factory_Init_inner(const MsgInner v) {
            auto oo = OOEnum_Val(OOEnum_Val::OOEnum::Inner);
            oo.oo.inner = v;
            // ReSharper disable once CppSomeObjectMembersMightNotBeInitialized
            return oo;
        };
        OOEnum_Val OOEnum_Val_Factory_Init_float_val(const BitisFloatingPoint<float> v) {
            auto oo = OOEnum_Val(OOEnum_Val::OOEnum::FloatVal);
            oo.oo.float_val = v;
            // ReSharper disable once CppSomeObjectMembersMightNotBeInitialized
            return oo;
        };
        OOEnum_Val OOEnum_Val_Factory_Init_small_int(const IntgralWithGivenBitSize<uint8_t, 4> v) {
            auto oo = OOEnum_Val(OOEnum_Val::OOEnum::SmallInt);
            oo.oo.small_int = v;
            // ReSharper disable once CppSomeObjectMembersMightNotBeInitialized
            return oo;
        };
    }
    struct MsgC {

        static const char *msg_attr[];
        // static constexpr const char* msg_attr[] = {"a", "val", "b"};

        typedef message_helper::MessageT<
            IntgralWithGivenBitSize<uint16_t, 3>,
            MsgC_helper::OOEnum_Val,
            IntgralWithGivenBitSize<uint16_t, 3>
        > MsgT;

        IntgralWithGivenBitSize<uint16_t, 3> a;
        MsgC_helper::OOEnum_Val val;
        IntgralWithGivenBitSize<uint16_t, 3> b;

        std::size_t serialize(BitisSerializer &ser) const {
            return message_helper::msg_serialize(this, ser);
        }
        void print(int16_t indent=0) {
            print_indent(indent); printf("MsgC{ ");
            if (indent>=0) printf("\n");
            message_helper::msg_print(this, indent + 2, msg_attr);
            print_indent(indent); printf("}");
            // if (indent>=0) printf("\n");
        }
    };
    const char *MsgC::msg_attr[] = {"a", "val", "b"};

    TEST(BITIS_OneOf, message_a) {
        {
            auto d = MsgC{
                .a = IntgralWithGivenBitSize<uint16_t, 3>(3),
                .val = MsgC_helper::OOEnum_Val_Factory_Init_float_val(BitisFloatingPoint<float>(1.2345)),
                .b = IntgralWithGivenBitSize<uint16_t, 3>(5),
            };
            d.print(0);
            printf("\n");

            auto ser = BitisSerializer();
            // auto r = serialize(a, ser);
            const auto r = d.serialize(ser);
            printf("r: %" PRIu64 "\n", r);
        }
        {
            auto d = MsgC{
                .a = IntgralWithGivenBitSize<uint16_t, 3>(2),
                .val = MsgC_helper::OOEnum_Val_Factory_Init_small_int(IntgralWithGivenBitSize<uint8_t, 4>(7)),
                .b = IntgralWithGivenBitSize<uint16_t, 3>(1),
            };
            d.print(0);
            printf("\n");

            auto ser = BitisSerializer();
            // auto r = serialize(a, ser);
            const auto r = d.serialize(ser);
            printf("r: %" PRIu64 "\n", r);
        }
        {
            auto d = MsgC{
                .a = IntgralWithGivenBitSize<uint16_t, 3>(2),
                .val = MsgC_helper::OOEnum_Val_Factory_Init_inner(MsgInner{.a = IntgralWithGivenBitSize<uint16_t, 12>(2040)}),
                .b = IntgralWithGivenBitSize<uint16_t, 3>(1),
            };
            d.print(0);
            printf("\n");

            auto ser = BitisSerializer();
            // auto r = serialize(a, ser);
            const auto r = d.serialize(ser);
            printf("r: %" PRIu64 "\n", r);
        }
    }
}


