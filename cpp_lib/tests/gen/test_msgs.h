#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_test_msgs::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_test_msgs {
    constexpr const char* BITIS_VERSION = "0.11.1";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0xe5d660b233c29b6cULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_test_msgs::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_test_msgs::BITIS_SCHEMA_HASH;
#endif



// ****** Mode *****
namespace ModeEnum {
    ENUM_INSTANCE(Off);
    ENUM_INSTANCE(On);
    ENUM_INSTANCE(Standby);
}

typedef BitisEnum<bitis_helper::Collector<
    ModeEnum::Off, 
    ModeEnum::On, 
    ModeEnum::Standby
>, ModeEnum::On, 2> Mode;



// ****** Color *****
namespace ColorEnum {
    ENUM_INSTANCE(Red);
    ENUM_INSTANCE(Green);
    ENUM_INSTANCE(Blue);
    ENUM_INSTANCE(Cyan);
    ENUM_INSTANCE(Magenta);
    ENUM_INSTANCE(Yellow);
}

typedef BitisEnum<bitis_helper::Collector<
    ColorEnum::Red, 
    ColorEnum::Green, 
    ColorEnum::Blue, 
    ColorEnum::Cyan, 
    ColorEnum::Magenta, 
    ColorEnum::Yellow
>, ColorEnum::Green, 2> Color;



// ****** Point *****


struct Point {
    typedef IntgralWithGivenBitSize<int16_t, 16> Px_T;
    typedef IntgralWithGivenBitSize<int32_t, 32> Py_T;
    typedef IntgralWithGivenBitSize<int64_t, 64> Pz_T;
    typedef IntgralWithGivenBitSize<uint64_t, 64> Pw_T;

    typedef message_helper::MessageT<
        Px_T, Py_T, Pz_T, Pw_T
    > MsgT;

    Px_T px;
    Py_T py;
    Pz_T pz;
    Pw_T pw;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Point>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Point>(); }
    static bitis_helper::BitiaDeserializerHelper<Point> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Point>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[4] = {"px", "py", "pz", "pw"};
    void print(int16_t indent=0) const {
        printf("Point{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Point &other) const {
        return px==other.px && py==other.py && pz==other.pz && pw==other.pw;}
    bool operator==(const Point &other) const { return is_equal(other); }
    bool operator!=(const Point &other) const { return !is_equal(other); }
};

// ****** Empty *****


struct Empty {

    typedef message_helper::MessageT<
        
    > MsgT;


    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Empty>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Empty>(); }
    static bitis_helper::BitiaDeserializerHelper<Empty> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Empty>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[0 + 1] = {};
    void print(int16_t indent=0) const {
        printf("Empty{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Empty &other) const {
        (void) other; return true;}
    bool operator==(const Empty &other) const { return is_equal(other); }
    bool operator!=(const Empty &other) const { return !is_equal(other); }
};

// ****** OO_AllTypes_Payload *****


struct OO_AllTypes_Payload  {
    struct OO_Flag {
        BITIS_NAME("Flag") typedef BitisBool OOType; };
    struct OO_Text {
        BITIS_NAME("Text") typedef BitisAString<3> OOType; };
    struct OO_Num {
        BITIS_NAME("Num") typedef BitisFloatingPoint<double> OOType; };
    struct OO_Pt {
        BITIS_NAME("Pt") typedef Point OOType; };
    struct OO_Nothing {
        BITIS_NAME("Nothing") typedef Empty OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Flag, 
        OO_Text, 
        OO_Num, 
        OO_Pt, 
        OO_Nothing
    >, OO_Flag, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Flag::OOType, OO_Text::OOType, OO_Num::OOType, OO_Pt::OOType, OO_Nothing::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_AllTypes_Payload() : oo_selector() {}

    template<typename OOT>
    OO_AllTypes_Payload set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
        return nullptr;
    }
    template<typename OOT>
    bool is_oo_value() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return true;
        return false;
    }

    std::size_t serialize(BitisSerializer &ser) const {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_AllTypes_Payload>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_AllTypes_Payload>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_AllTypes_Payload> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_AllTypes_Payload>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_AllTypes_Payload &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_AllTypes_Payload &other) const { return is_equal(other); }
    bool operator!=(const OO_AllTypes_Payload &other) const { return !is_equal(other); }
};


// ****** AllTypes *****


struct AllTypes {
    typedef BitisBool Flag_T;
    typedef IntgralWithGivenBitSize<uint16_t, 3> U3_T;
    typedef IntgralWithGivenBitSize<int16_t, 5> I5_T;
    typedef IntgralWithGivenBitSize<int16_t, 16> I16_T;
    typedef DynInteger<uint32_t, 32, 4> Ud_T;
    typedef DynInteger<int16_t, 16, 3> Sd_T;
    typedef BitisFloatingPoint<float> Fl_T;
    typedef BitisFloatingPoint<double> Db_T;
    typedef FixPrecisionMinMax<10, -2, 3> Fp_T;
    typedef BitisOptional<IntgralWithGivenBitSize<uint16_t, 7>> Ou_T;
    typedef FixedArray<IntgralWithGivenBitSize<int16_t, 4>,2> Fa_T;
    typedef DynArray<IntgralWithGivenBitSize<uint16_t, 6>,3> Da_T;
    typedef Mode Mode_T;
    typedef BitisAString<4> Name_T;
    typedef DynArray<Point,2> Points_T;
    typedef OO_AllTypes_Payload Payload_T;

    typedef message_helper::MessageT<
        Flag_T, U3_T, I5_T, I16_T, Ud_T, Sd_T, Fl_T, Db_T, Fp_T, Ou_T, Fa_T, Da_T, Mode_T, Name_T, Points_T, Payload_T
    > MsgT;

    Flag_T flag;
    U3_T u3;
    I5_T i5;
    I16_T i16;
    Ud_T ud;
    Sd_T sd;
    Fl_T fl;
    Db_T db;
    Fp_T fp;
    Ou_T ou;
    Fa_T fa;
    Da_T da;
    Mode_T mode;
    Name_T name;
    Points_T points;
    Payload_T payload;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<AllTypes>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<AllTypes>(); }
    static bitis_helper::BitiaDeserializerHelper<AllTypes> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<AllTypes>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[16] = {"flag", "u3", "i5", "i16", "ud", "sd", "fl", "db", "fp", "ou", "fa", "da", "mode", "name", "points", "payload"};
    void print(int16_t indent=0) const {
        printf("AllTypes{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const AllTypes &other) const {
        return flag==other.flag && u3==other.u3 && i5==other.i5 && i16==other.i16 && ud==other.ud && sd==other.sd && fl==other.fl && db==other.db && fp==other.fp && ou==other.ou && fa==other.fa && da==other.da && mode==other.mode && name==other.name && points==other.points && payload==other.payload;}
    bool operator==(const AllTypes &other) const { return is_equal(other); }
    bool operator!=(const AllTypes &other) const { return !is_equal(other); }
};

// ****** Inner *****


struct Inner {
    typedef IntgralWithGivenBitSize<int16_t, 5> Va_T;
    typedef BitisBool Vb_T;

    typedef message_helper::MessageT<
        Va_T, Vb_T
    > MsgT;

    Va_T va;
    Vb_T vb;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Inner>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Inner>(); }
    static bitis_helper::BitiaDeserializerHelper<Inner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Inner>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"va", "vb"};
    void print(int16_t indent=0) const {
        printf("Inner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Inner &other) const {
        return va==other.va && vb==other.vb;}
    bool operator==(const Inner &other) const { return is_equal(other); }
    bool operator!=(const Inner &other) const { return !is_equal(other); }
};

// ****** OO_Xl_Val *****


struct OO_Xl_Val  {
    struct OO_Flag {
        BITIS_NAME("Flag") typedef BitisBool OOType; };
    struct OO_Num {
        BITIS_NAME("Num") typedef BitisFloatingPoint<double> OOType; };
    struct OO_Inner {
        BITIS_NAME("Inner") typedef Inner OOType; };
    struct OO_Big {
        BITIS_NAME("Big") typedef DynInteger<int32_t, 32, 7> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Flag, 
        OO_Num, 
        OO_Inner, 
        OO_Big
    >, OO_Flag, 3> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Flag::OOType, OO_Num::OOType, OO_Inner::OOType, OO_Big::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_Xl_Val() : oo_selector() {}

    template<typename OOT>
    OO_Xl_Val set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
        return nullptr;
    }
    template<typename OOT>
    bool is_oo_value() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return true;
        return false;
    }

    std::size_t serialize(BitisSerializer &ser) const {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Xl_Val>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_Xl_Val>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Xl_Val> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Xl_Val>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_Xl_Val &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Xl_Val &other) const { return is_equal(other); }
    bool operator!=(const OO_Xl_Val &other) const { return !is_equal(other); }
};


// ****** XL *****


struct XL {
    typedef IntgralWithGivenBitSize<uint16_t, 3> U3_T;
    typedef IntgralWithGivenBitSize<int16_t, 5> I5_T;
    typedef IntgralWithGivenBitSize<int16_t, 15> I15_T;
    typedef IntgralWithGivenBitSize<uint16_t, 13> U13_T;
    typedef DynInteger<uint16_t, 4, 3> Ud_T;
    typedef DynInteger<int16_t, 12, 4> Sd_T;
    typedef BitisFloatingPoint<float> Fl_T;
    typedef BitisFloatingPoint<double> Db_T;
    typedef FixPrecisionMinMax<10, -2, 3> Fp_T;
    typedef FixPrecisionMinMax<12, -2000, 18000> Fp2_T;
    typedef BitisOptional<IntgralWithGivenBitSize<int16_t, 7>> Oi_T;
    typedef BitisOptional<Inner> Oinner_T;
    typedef FixedArray<IntgralWithGivenBitSize<int16_t, 4>,3> Fa_T;
    typedef DynArray<IntgralWithGivenBitSize<uint16_t, 6>,3> Da_T;
    typedef Color Col_T;
    typedef BitisAString<4> Str_T;
    typedef OO_Xl_Val Val_T;

    typedef message_helper::MessageT<
        U3_T, I5_T, I15_T, U13_T, Ud_T, Sd_T, Fl_T, Db_T, Fp_T, Fp2_T, Oi_T, Oinner_T, Fa_T, Da_T, Col_T, Str_T, Val_T
    > MsgT;

    U3_T u3;
    I5_T i5;
    I15_T i15;
    U13_T u13;
    Ud_T ud;
    Sd_T sd;
    Fl_T fl;
    Db_T db;
    Fp_T fp;
    Fp2_T fp2;
    Oi_T oi;
    Oinner_T oinner;
    Fa_T fa;
    Da_T da;
    Col_T col;
    Str_T str;
    Val_T val;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<XL>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<XL>(); }
    static bitis_helper::BitiaDeserializerHelper<XL> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<XL>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[17] = {"u3", "i5", "i15", "u13", "ud", "sd", "fl", "db", "fp", "fp2", "oi", "oinner", "fa", "da", "col", "str", "val"};
    void print(int16_t indent=0) const {
        printf("XL{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const XL &other) const {
        return u3==other.u3 && i5==other.i5 && i15==other.i15 && u13==other.u13 && ud==other.ud && sd==other.sd && fl==other.fl && db==other.db && fp==other.fp && fp2==other.fp2 && oi==other.oi && oinner==other.oinner && fa==other.fa && da==other.da && col==other.col && str==other.str && val==other.val;}
    bool operator==(const XL &other) const { return is_equal(other); }
    bool operator!=(const XL &other) const { return !is_equal(other); }
};

// ****** MsgKVSimple *****


struct MsgKVSimple {
    typedef BitisAString<4> Key_T;
    typedef BitisAString<4> Value_T;

    typedef message_helper::MessageT<
        Key_T, Value_T
    > MsgT;

    Key_T key;
    Value_T value;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVSimple>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<MsgKVSimple>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVSimple> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVSimple>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"key", "value"};
    void print(int16_t indent=0) const {
        printf("MsgKVSimple{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const MsgKVSimple &other) const {
        return key==other.key && value==other.value;}
    bool operator==(const MsgKVSimple &other) const { return is_equal(other); }
    bool operator!=(const MsgKVSimple &other) const { return !is_equal(other); }
};

// ****** MsgKVMapSimple *****


struct MsgKVMapSimple {
    typedef DynArray<MsgKVSimple,4> Entries_T;

    typedef message_helper::MessageT<
        Entries_T
    > MsgT;

    Entries_T entries;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVMapSimple>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<MsgKVMapSimple>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVMapSimple> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVMapSimple>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[1] = {"entries"};
    void print(int16_t indent=0) const {
        printf("MsgKVMapSimple{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const MsgKVMapSimple &other) const {
        return entries==other.entries;}
    bool operator==(const MsgKVMapSimple &other) const { return is_equal(other); }
    bool operator!=(const MsgKVMapSimple &other) const { return !is_equal(other); }
};

// ****** OO_MsgKvoo_Value *****


struct OO_MsgKvoo_Value  {
    struct OO_StrVal {
        BITIS_NAME("StrVal") typedef BitisAString<4> OOType; };
    struct OO_NumVal {
        BITIS_NAME("NumVal") typedef BitisFloatingPoint<double> OOType; };
    struct OO_BoolVal {
        BITIS_NAME("BoolVal") typedef BitisBool OOType; };
    struct OO_IntVal {
        BITIS_NAME("IntVal") typedef DynInteger<int32_t, 32, 7> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_StrVal, 
        OO_NumVal, 
        OO_BoolVal, 
        OO_IntVal
    >, OO_BoolVal, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_StrVal::OOType, OO_NumVal::OOType, OO_BoolVal::OOType, OO_IntVal::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_MsgKvoo_Value() : oo_selector() {}

    template<typename OOT>
    OO_MsgKvoo_Value set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
        return nullptr;
    }
    template<typename OOT>
    bool is_oo_value() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return true;
        return false;
    }

    std::size_t serialize(BitisSerializer &ser) const {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_MsgKvoo_Value>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_MsgKvoo_Value>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_MsgKvoo_Value> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_MsgKvoo_Value>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_MsgKvoo_Value &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_MsgKvoo_Value &other) const { return is_equal(other); }
    bool operator!=(const OO_MsgKvoo_Value &other) const { return !is_equal(other); }
};


// ****** MsgKVOO *****


struct MsgKVOO {
    typedef BitisAString<4> Key_T;
    typedef OO_MsgKvoo_Value Value_T;

    typedef message_helper::MessageT<
        Key_T, Value_T
    > MsgT;

    Key_T key;
    Value_T value;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVOO>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<MsgKVOO>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVOO> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVOO>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"key", "value"};
    void print(int16_t indent=0) const {
        printf("MsgKVOO{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const MsgKVOO &other) const {
        return key==other.key && value==other.value;}
    bool operator==(const MsgKVOO &other) const { return is_equal(other); }
    bool operator!=(const MsgKVOO &other) const { return !is_equal(other); }
};

// ****** MsgKVMapOO *****


struct MsgKVMapOO {
    typedef DynArray<MsgKVOO,2> Entries_T;

    typedef message_helper::MessageT<
        Entries_T
    > MsgT;

    Entries_T entries;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVMapOO>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<MsgKVMapOO>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVMapOO> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVMapOO>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[1] = {"entries"};
    void print(int16_t indent=0) const {
        printf("MsgKVMapOO{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const MsgKVMapOO &other) const {
        return entries==other.entries;}
    bool operator==(const MsgKVMapOO &other) const { return is_equal(other); }
    bool operator!=(const MsgKVMapOO &other) const { return !is_equal(other); }
};