#include "bitis_lib.h"
#include <optional>



// ****** SensorSource *****
namespace SensorSourceEnum {
    ENUM_INSTANCE(TemperaturSensor);
    ENUM_INSTANCE(MovementSensor);
}

typedef BitisEnum<bitis_helper::Collector<
    SensorSourceEnum::TemperaturSensor, 
    SensorSourceEnum::MovementSensor
>, SensorSourceEnum::TemperaturSensor, 3> SensorSource;



// ****** Inner *****


struct Inner {
    typedef IntgralWithGivenBitSize<int8_t, 3> Val2_T;

    typedef message_helper::MessageT<
        Val2_T
    > MsgT;

    Val2_T val2;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<Inner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Inner>(des);
    }

    void print(int16_t indent=0) {
        printf("Inner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const Inner &other) const {
        return val2==other.val2;
   }
    bool operator==(const Inner &other) const { return is_equal(other); }
    bool operator!=(const Inner &other) const { return !is_equal(other); }
};
const char *Inner::msg_attr[] = {"val2"};

// ****** ExampleEnum *****
namespace ExampleEnumEnum {
    ENUM_INSTANCE(E1);
    ENUM_INSTANCE(E2);
    ENUM_INSTANCE(E3);
    ENUM_INSTANCE(E4);
    ENUM_INSTANCE(E5);
    ENUM_INSTANCE(E6);
    ENUM_INSTANCE(E7);
    ENUM_INSTANCE(E8);
    ENUM_INSTANCE(E9);
}

typedef BitisEnum<bitis_helper::Collector<
    ExampleEnumEnum::E1, 
    ExampleEnumEnum::E2, 
    ExampleEnumEnum::E3, 
    ExampleEnumEnum::E4, 
    ExampleEnumEnum::E5, 
    ExampleEnumEnum::E6, 
    ExampleEnumEnum::E7, 
    ExampleEnumEnum::E8, 
    ExampleEnumEnum::E9
>, ExampleEnumEnum::E3, 2> ExampleEnum;



// ****** MsgFixedBaseArray *****


struct MsgFixedBaseArray {
    typedef SensorSource Param1_T;
    typedef FixedArray<IntgralWithGivenBitSize<uint8_t, 3>,3> Val1_T;
    typedef FixedArray<IntgralWithGivenBitSize<int8_t, 3>,3> Val2_T;
    typedef FixedArray<BitisBool,3> Val3_T;
    typedef FixedArray<DynInteger<int8_t, 3>,3> Val4_T;
    typedef FixedArray<BitisFloatingPoint<double>,3> Val5_T;
    typedef FixedArray<FixPrecisionMinMax<10, -2, 3>,3> Val6_T;
    typedef FixedArray<SensorSource,3> Val7_T;
    typedef FixedArray<Inner,3> Val8_T;

    typedef message_helper::MessageT<
        Param1_T, Val1_T, Val2_T, Val3_T, Val4_T, Val5_T, Val6_T, Val7_T, Val8_T
    > MsgT;

    Param1_T param_1;
    Val1_T val1;
    Val2_T val2;
    Val3_T val3;
    Val4_T val4;
    Val5_T val5;
    Val6_T val6;
    Val7_T val7;
    Val8_T val8;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgFixedBaseArray> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgFixedBaseArray>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgFixedBaseArray{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgFixedBaseArray &other) const {
        return param_1==other.param_1 && val1==other.val1 && val2==other.val2 && val3==other.val3 && val4==other.val4 && val5==other.val5 && val6==other.val6 && val7==other.val7 && val8==other.val8;
   }
    bool operator==(const MsgFixedBaseArray &other) const { return is_equal(other); }
    bool operator!=(const MsgFixedBaseArray &other) const { return !is_equal(other); }
};
const char *MsgFixedBaseArray::msg_attr[] = {"param_1", "val1", "val2", "val3", "val4", "val5", "val6", "val7", "val8"};

// ****** MsgDynBaseArray *****


struct MsgDynBaseArray {
    typedef ExampleEnum Ee_T;
    typedef DynArray<IntgralWithGivenBitSize<uint8_t, 3>,3> Val1_T;
    typedef DynArray<IntgralWithGivenBitSize<int8_t, 3>,3> Val2_T;
    typedef DynArray<BitisBool,3> Val3_T;
    typedef DynArray<DynInteger<int8_t, 3>,3> Val4_T;
    typedef DynArray<BitisFloatingPoint<double>,3> Val5_T;
    typedef DynArray<FixPrecisionMinMax<10, -2, 3>,3> Val6_T;
    typedef DynArray<SensorSource,6> Val7_T;
    typedef DynArray<Inner,3> Val8_T;

    typedef message_helper::MessageT<
        Ee_T, Val1_T, Val2_T, Val3_T, Val4_T, Val5_T, Val6_T, Val7_T, Val8_T
    > MsgT;

    Ee_T ee;
    Val1_T val1;
    Val2_T val2;
    Val3_T val3;
    Val4_T val4;
    Val5_T val5;
    Val6_T val6;
    Val7_T val7;
    Val8_T val8;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgDynBaseArray> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgDynBaseArray>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgDynBaseArray{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgDynBaseArray &other) const {
        return ee==other.ee && val1==other.val1 && val2==other.val2 && val3==other.val3 && val4==other.val4 && val5==other.val5 && val6==other.val6 && val7==other.val7 && val8==other.val8;
   }
    bool operator==(const MsgDynBaseArray &other) const { return is_equal(other); }
    bool operator!=(const MsgDynBaseArray &other) const { return !is_equal(other); }
};
const char *MsgDynBaseArray::msg_attr[] = {"ee", "val1", "val2", "val3", "val4", "val5", "val6", "val7", "val8"};

// ****** MsgLargeFixedArray *****


struct MsgLargeFixedArray {
    typedef SensorSource Param1_T;
    typedef FixedArray<IntgralWithGivenBitSize<uint8_t, 3>,100> Val1_T;
    typedef FixedArray<IntgralWithGivenBitSize<int8_t, 3>,100> Val2_T;
    typedef FixedArray<BitisBool,100> Val3_T;

    typedef message_helper::MessageT<
        Param1_T, Val1_T, Val2_T, Val3_T
    > MsgT;

    Param1_T param_1;
    Val1_T val1;
    Val2_T val2;
    Val3_T val3;

    static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgLargeFixedArray> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgLargeFixedArray>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgLargeFixedArray{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgLargeFixedArray &other) const {
        return param_1==other.param_1 && val1==other.val1 && val2==other.val2 && val3==other.val3;
   }
    bool operator==(const MsgLargeFixedArray &other) const { return is_equal(other); }
    bool operator!=(const MsgLargeFixedArray &other) const { return !is_equal(other); }
};
const char *MsgLargeFixedArray::msg_attr[] = {"param_1", "val1", "val2", "val3"};