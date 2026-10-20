#pragma once

#include "bitis_lib.h"
#include <optional>

//#define EXPECTED_BITIS_VERSION "0.10.5"
//#if EXPECTED_BITIS_VERSION != BITIS_CPP_LIB_VERSION
//#error "Unexpected bitis library version"
//#endif



// ****** Empty *****


struct Empty {

    typedef message_helper::MessageT<
        
    > MsgT;

    const char *msg_attr[0 + 1] = {};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Empty>(); }
    static bitis_helper::BitiaDeserializerHelper<Empty> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Empty>(des);
    }

    void print(int16_t indent=0) const {
        printf("Empty{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const Empty &other) const {
        (void) other; return true;}
    bool operator==(const Empty &other) const { return is_equal(other); }
    bool operator!=(const Empty &other) const { return !is_equal(other); }
};
//const char *Empty::msg_attr[] = {};

// ****** MsgKVSimple *****


struct MsgKVSimple {
    typedef BitisAString<4> Key_T;
    typedef BitisAString<4> Value_T;

    typedef message_helper::MessageT<
        Key_T, Value_T
    > MsgT;

    Key_T key;
    Value_T value;
    const char * str_key = "key";
    const char * str_value = "value";
    const char *msg_attr[2] = {str_key, str_value};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVSimple>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVSimple> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVSimple>(des);
    }

    void print(int16_t indent=0) const {
        printf("MsgKVSimple{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVSimple &other) const {
        return key==other.key && value==other.value;}
    bool operator==(const MsgKVSimple &other) const { return is_equal(other); }
    bool operator!=(const MsgKVSimple &other) const { return !is_equal(other); }
};
//const char *MsgKVSimple::msg_attr[] = {"key", "value"};

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



// ****** Inner *****


struct Inner {
    typedef IntgralWithGivenBitSize<int16_t, 5> Va_T;
    typedef BitisBool Vb_T;

    typedef message_helper::MessageT<
        Va_T, Vb_T
    > MsgT;

    Va_T va;
    Vb_T vb;
    const char * str_va = "va";
    const char * str_vb = "vb";
    const char *msg_attr[2] = {str_va, str_vb};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Inner>(); }
    static bitis_helper::BitiaDeserializerHelper<Inner> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Inner>(des);
    }

    void print(int16_t indent=0) const {
        printf("Inner{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const Inner &other) const {
        return va==other.va && vb==other.vb;}
    bool operator==(const Inner &other) const { return is_equal(other); }
    bool operator!=(const Inner &other) const { return !is_equal(other); }
};
//const char *Inner::msg_attr[] = {"va", "vb"};

// ****** OO_MsgKvoo_Value *****


struct OO_MsgKvoo_Value  {
    struct OO_StrVal {
        static constexpr auto name = "StrVal"; typedef BitisAString<4> OOType; };
    struct OO_NumVal {
        static constexpr auto name = "NumVal"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_BoolVal {
        static constexpr auto name = "BoolVal"; typedef BitisBool OOType; };
    struct OO_IntVal {
        static constexpr auto name = "IntVal"; typedef DynInteger<int32_t, 32, 7> OOType; };

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
    //uint8_t *oo_value_ptr;

    OO_MsgKvoo_Value() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_MsgKvoo_Value() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_MsgKvoo_Value set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;

        //delete [] oo_value_ptr;
        //oo_value_ptr = new uint8_t[sizeof(typename OOT::OOType)];
        //memcpy(oo_value_ptr, &v, sizeof(v));
        //return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
            //return (typename OOT::OOType*) oo_value_ptr;
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
    static bitis_helper::BitiaDeserializerHelper<OO_MsgKvoo_Value> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_MsgKvoo_Value>(des);
    }

    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_MsgKvoo_Value &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_MsgKvoo_Value &other) const { return is_equal(other); }
    bool operator!=(const OO_MsgKvoo_Value &other) const { return !is_equal(other); }
};


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
    const char * str_px = "px";
    const char * str_py = "py";
    const char * str_pz = "pz";
    const char * str_pw = "pw";
    const char *msg_attr[4] = {str_px, str_py, str_pz, str_pw};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Point>(); }
    static bitis_helper::BitiaDeserializerHelper<Point> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Point>(des);
    }

    void print(int16_t indent=0) const {
        printf("Point{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const Point &other) const {
        return px==other.px && py==other.py && pz==other.pz && pw==other.pw;}
    bool operator==(const Point &other) const { return is_equal(other); }
    bool operator!=(const Point &other) const { return !is_equal(other); }
};
//const char *Point::msg_attr[] = {"px", "py", "pz", "pw"};

// ****** MsgKVMapSimple *****


struct MsgKVMapSimple {
    typedef DynArray<MsgKVSimple,4> Entries_T;

    typedef message_helper::MessageT<
        Entries_T
    > MsgT;

    Entries_T entries;
    const char * str_entries = "entries";
    const char *msg_attr[1] = {str_entries};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVMapSimple>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVMapSimple> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVMapSimple>(des);
    }

    void print(int16_t indent=0) const {
        printf("MsgKVMapSimple{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVMapSimple &other) const {
        return entries==other.entries;}
    bool operator==(const MsgKVMapSimple &other) const { return is_equal(other); }
    bool operator!=(const MsgKVMapSimple &other) const { return !is_equal(other); }
};
//const char *MsgKVMapSimple::msg_attr[] = {"entries"};

// ****** MsgKVOO *****


struct MsgKVOO {
    typedef BitisAString<4> Key_T;
    typedef OO_MsgKvoo_Value Value_T;

    typedef message_helper::MessageT<
        Key_T, Value_T
    > MsgT;

    Key_T key;
    Value_T value;
    const char * str_key = "key";
    const char * str_value = "value";
    const char *msg_attr[2] = {str_key, str_value};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVOO>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVOO> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVOO>(des);
    }

    void print(int16_t indent=0) const {
        printf("MsgKVOO{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVOO &other) const {
        return key==other.key && value==other.value;}
    bool operator==(const MsgKVOO &other) const { return is_equal(other); }
    bool operator!=(const MsgKVOO &other) const { return !is_equal(other); }
};
//const char *MsgKVOO::msg_attr[] = {"key", "value"};

// ****** OO_Xl_Val *****


struct OO_Xl_Val  {
    struct OO_Flag {
        static constexpr auto name = "Flag"; typedef BitisBool OOType; };
    struct OO_Num {
        static constexpr auto name = "Num"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_Inner {
        static constexpr auto name = "Inner"; typedef Inner OOType; };
    struct OO_Big {
        static constexpr auto name = "Big"; typedef DynInteger<int32_t, 32, 7> OOType; };

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
    //uint8_t *oo_value_ptr;

    OO_Xl_Val() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_Xl_Val() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_Xl_Val set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;

        //delete [] oo_value_ptr;
        //oo_value_ptr = new uint8_t[sizeof(typename OOT::OOType)];
        //memcpy(oo_value_ptr, &v, sizeof(v));
        //return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
            //return (typename OOT::OOType*) oo_value_ptr;
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
    static bitis_helper::BitiaDeserializerHelper<OO_Xl_Val> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Xl_Val>(des);
    }

    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_Xl_Val &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Xl_Val &other) const { return is_equal(other); }
    bool operator!=(const OO_Xl_Val &other) const { return !is_equal(other); }
};


// ****** OO_AllTypes_Payload *****


struct OO_AllTypes_Payload  {
    struct OO_None {
        static constexpr auto name = "None"; typedef BitisBool OOType; };
    struct OO_Text {
        static constexpr auto name = "Text"; typedef BitisAString<3> OOType; };
    struct OO_Num {
        static constexpr auto name = "Num"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_Pt {
        static constexpr auto name = "Pt"; typedef Point OOType; };
    struct OO_Nothing {
        static constexpr auto name = "Nothing"; typedef Empty OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_None, 
        OO_Text, 
        OO_Num, 
        OO_Pt, 
        OO_Nothing
    >, OO_None, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_None::OOType, OO_Text::OOType, OO_Num::OOType, OO_Pt::OOType, OO_Nothing::OOType
    > T_OOValue;
    T_OOValue oo_value;
    //uint8_t *oo_value_ptr;

    OO_AllTypes_Payload() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_AllTypes_Payload() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_AllTypes_Payload set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;

        //delete [] oo_value_ptr;
        //oo_value_ptr = new uint8_t[sizeof(typename OOT::OOType)];
        //memcpy(oo_value_ptr, &v, sizeof(v));
        //return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        if(oo_selector.is_enum<OOT>())
            return oo_value.get<typename OOT::OOType>();
            //return (typename OOT::OOType*) oo_value_ptr;
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
    static bitis_helper::BitiaDeserializerHelper<OO_AllTypes_Payload> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_AllTypes_Payload>(des);
    }

    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_AllTypes_Payload &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_AllTypes_Payload &other) const { return is_equal(other); }
    bool operator!=(const OO_AllTypes_Payload &other) const { return !is_equal(other); }
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
    const char * str_u3 = "u3";
    const char * str_i5 = "i5";
    const char * str_i15 = "i15";
    const char * str_u13 = "u13";
    const char * str_ud = "ud";
    const char * str_sd = "sd";
    const char * str_fl = "fl";
    const char * str_db = "db";
    const char * str_fp = "fp";
    const char * str_fp2 = "fp2";
    const char * str_oi = "oi";
    const char * str_oinner = "oinner";
    const char * str_fa = "fa";
    const char * str_da = "da";
    const char * str_col = "col";
    const char * str_str = "str";
    const char * str_val = "val";
    const char *msg_attr[17] = {str_u3, str_i5, str_i15, str_u13, str_ud, str_sd, str_fl, str_db, str_fp, str_fp2, str_oi, str_oinner, str_fa, str_da, str_col, str_str, str_val};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<XL>(); }
    static bitis_helper::BitiaDeserializerHelper<XL> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<XL>(des);
    }

    void print(int16_t indent=0) const {
        printf("XL{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const XL &other) const {
        return u3==other.u3 && i5==other.i5 && i15==other.i15 && u13==other.u13 && ud==other.ud && sd==other.sd && fl==other.fl && db==other.db && fp==other.fp && fp2==other.fp2 && oi==other.oi && oinner==other.oinner && fa==other.fa && da==other.da && col==other.col && str==other.str && val==other.val;}
    bool operator==(const XL &other) const { return is_equal(other); }
    bool operator!=(const XL &other) const { return !is_equal(other); }
};
//const char *XL::msg_attr[] = {"u3", "i5", "i15", "u13", "ud", "sd", "fl", "db", "fp", "fp2", "oi", "oinner", "fa", "da", "col", "str", "val"};

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
    const char * str_flag = "flag";
    const char * str_u3 = "u3";
    const char * str_i5 = "i5";
    const char * str_i16 = "i16";
    const char * str_ud = "ud";
    const char * str_sd = "sd";
    const char * str_fl = "fl";
    const char * str_db = "db";
    const char * str_fp = "fp";
    const char * str_ou = "ou";
    const char * str_fa = "fa";
    const char * str_da = "da";
    const char * str_mode = "mode";
    const char * str_name = "name";
    const char * str_points = "points";
    const char * str_payload = "payload";
    const char *msg_attr[16] = {str_flag, str_u3, str_i5, str_i16, str_ud, str_sd, str_fl, str_db, str_fp, str_ou, str_fa, str_da, str_mode, str_name, str_points, str_payload};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<AllTypes>(); }
    static bitis_helper::BitiaDeserializerHelper<AllTypes> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<AllTypes>(des);
    }

    void print(int16_t indent=0) const {
        printf("AllTypes{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const AllTypes &other) const {
        return flag==other.flag && u3==other.u3 && i5==other.i5 && i16==other.i16 && ud==other.ud && sd==other.sd && fl==other.fl && db==other.db && fp==other.fp && ou==other.ou && fa==other.fa && da==other.da && mode==other.mode && name==other.name && points==other.points && payload==other.payload;}
    bool operator==(const AllTypes &other) const { return is_equal(other); }
    bool operator!=(const AllTypes &other) const { return !is_equal(other); }
};
//const char *AllTypes::msg_attr[] = {"flag", "u3", "i5", "i16", "ud", "sd", "fl", "db", "fp", "ou", "fa", "da", "mode", "name", "points", "payload"};

// ****** MsgKVMapOO *****


struct MsgKVMapOO {
    typedef DynArray<MsgKVOO,2> Entries_T;

    typedef message_helper::MessageT<
        Entries_T
    > MsgT;

    Entries_T entries;
    const char * str_entries = "entries";
    const char *msg_attr[1] = {str_entries};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<MsgKVMapOO>(); }
    static bitis_helper::BitiaDeserializerHelper<MsgKVMapOO> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgKVMapOO>(des);
    }

    void print(int16_t indent=0) const {
        printf("MsgKVMapOO{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgKVMapOO &other) const {
        return entries==other.entries;}
    bool operator==(const MsgKVMapOO &other) const { return is_equal(other); }
    bool operator!=(const MsgKVMapOO &other) const { return !is_equal(other); }
};
//const char *MsgKVMapOO::msg_attr[] = {"entries"};