#pragma once

#include "bitis_lib.h"
#include <optional>
#include <cstdint>

// ****** schema stamp *****
// Compiler version and schema hash (first 8 bytes of SHA-256 over the canonical schema, see bitis
// docs/WIRE_FORMAT.md). Always available as bitis_schema_golden::BITIS_VERSION / ::BITIS_SCHEMA_HASH;
// the unqualified names refer to the first generated header included in a translation unit.
namespace bitis_schema_golden {
    constexpr const char* BITIS_VERSION = "0.10.5";
    constexpr uint64_t BITIS_SCHEMA_HASH = 0xb637cfeeccc2d168ULL;
}
#ifndef BITIS_SCHEMA_STAMP_DEFINED
#define BITIS_SCHEMA_STAMP_DEFINED
constexpr const char* BITIS_VERSION = bitis_schema_golden::BITIS_VERSION;
constexpr uint64_t BITIS_SCHEMA_HASH = bitis_schema_golden::BITIS_SCHEMA_HASH;
#endif



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



// ****** Level *****
namespace LevelEnum {
    ENUM_INSTANCE(Low);
    ENUM_INSTANCE(Mid);
    ENUM_INSTANCE(High);
}

typedef BitisEnum<bitis_helper::Collector<
    LevelEnum::Low, 
    LevelEnum::Mid, 
    LevelEnum::High
>, LevelEnum::Low, 1> Level;



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

// ****** Ints *****


struct Ints {
    typedef BitisBool Flag_T;
    typedef IntgralWithGivenBitSize<uint16_t, 1> ValU1_T;
    typedef IntgralWithGivenBitSize<uint16_t, 7> ValU7_T;
    typedef IntgralWithGivenBitSize<uint32_t, 32> ValU32_T;
    typedef IntgralWithGivenBitSize<uint64_t, 33> ValU33_T;
    typedef IntgralWithGivenBitSize<uint64_t, 63> ValU63_T;
    typedef IntgralWithGivenBitSize<uint64_t, 64> ValU64_T;
    typedef IntgralWithGivenBitSize<int16_t, 1> ValI1_T;
    typedef IntgralWithGivenBitSize<int16_t, 7> ValI7_T;
    typedef IntgralWithGivenBitSize<int16_t, 16> ValI16_T;
    typedef IntgralWithGivenBitSize<int32_t, 32> ValI32_T;
    typedef IntgralWithGivenBitSize<int64_t, 33> ValI33_T;
    typedef IntgralWithGivenBitSize<int64_t, 64> ValI64_T;

    typedef message_helper::MessageT<
        Flag_T, ValU1_T, ValU7_T, ValU32_T, ValU33_T, ValU63_T, ValU64_T, ValI1_T, ValI7_T, ValI16_T, ValI32_T, ValI33_T, ValI64_T
    > MsgT;

    Flag_T flag;
    ValU1_T val_u1;
    ValU7_T val_u7;
    ValU32_T val_u32;
    ValU33_T val_u33;
    ValU63_T val_u63;
    ValU64_T val_u64;
    ValI1_T val_i1;
    ValI7_T val_i7;
    ValI16_T val_i16;
    ValI32_T val_i32;
    ValI33_T val_i33;
    ValI64_T val_i64;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Ints>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Ints>(); }
    static bitis_helper::BitiaDeserializerHelper<Ints> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Ints>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[13] = {"flag", "val_u1", "val_u7", "val_u32", "val_u33", "val_u63", "val_u64", "val_i1", "val_i7", "val_i16", "val_i32", "val_i33", "val_i64"};
    void print(int16_t indent=0) const {
        printf("Ints{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Ints &other) const {
        return flag==other.flag && val_u1==other.val_u1 && val_u7==other.val_u7 && val_u32==other.val_u32 && val_u33==other.val_u33 && val_u63==other.val_u63 && val_u64==other.val_u64 && val_i1==other.val_i1 && val_i7==other.val_i7 && val_i16==other.val_i16 && val_i32==other.val_i32 && val_i33==other.val_i33 && val_i64==other.val_i64;}
    bool operator==(const Ints &other) const { return is_equal(other); }
    bool operator!=(const Ints &other) const { return !is_equal(other); }
};

// ****** DynInts *****


struct DynInts {
    typedef DynInteger<uint16_t, 8, 3> ValU8D3_T;
    typedef DynInteger<uint32_t, 32, 4> ValU32D4_T;
    typedef DynInteger<uint64_t, 64, 64> ValU64D64_T;
    typedef DynInteger<uint64_t, 64, 7> ValU64D7_T;
    typedef DynInteger<int16_t, 16, 5> ValI16D5_T;
    typedef DynInteger<int64_t, 64, 8> ValI64D8_T;

    typedef message_helper::MessageT<
        ValU8D3_T, ValU32D4_T, ValU64D64_T, ValU64D7_T, ValI16D5_T, ValI64D8_T
    > MsgT;

    ValU8D3_T val_u8d3;
    ValU32D4_T val_u32d4;
    ValU64D64_T val_u64d64;
    ValU64D7_T val_u64d7;
    ValI16D5_T val_i16d5;
    ValI64D8_T val_i64d8;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<DynInts>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<DynInts>(); }
    static bitis_helper::BitiaDeserializerHelper<DynInts> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<DynInts>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[6] = {"val_u8d3", "val_u32d4", "val_u64d64", "val_u64d7", "val_i16d5", "val_i64d8"};
    void print(int16_t indent=0) const {
        printf("DynInts{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const DynInts &other) const {
        return val_u8d3==other.val_u8d3 && val_u32d4==other.val_u32d4 && val_u64d64==other.val_u64d64 && val_u64d7==other.val_u64d7 && val_i16d5==other.val_i16d5 && val_i64d8==other.val_i64d8;}
    bool operator==(const DynInts &other) const { return is_equal(other); }
    bool operator!=(const DynInts &other) const { return !is_equal(other); }
};

// ****** Floats *****


struct Floats {
    typedef BitisFloatingPoint<float> ValF32_T;
    typedef BitisFloatingPoint<double> ValF64_T;

    typedef message_helper::MessageT<
        ValF32_T, ValF64_T
    > MsgT;

    ValF32_T val_f32;
    ValF64_T val_f64;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Floats>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Floats>(); }
    static bitis_helper::BitiaDeserializerHelper<Floats> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Floats>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"val_f32", "val_f64"};
    void print(int16_t indent=0) const {
        printf("Floats{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Floats &other) const {
        return val_f32==other.val_f32 && val_f64==other.val_f64;}
    bool operator==(const Floats &other) const { return is_equal(other); }
    bool operator!=(const Floats &other) const { return !is_equal(other); }
};

// ****** FixedPoints *****


struct FixedPoints {
    typedef FixPrecisionMinMax<2, 0, 1> Fp2_T;
    typedef FixPrecisionMinMax<10, -2, 3> Fp10_T;
    typedef FixPrecisionMinMax<34, -90, 90> Lat_T;
    typedef FixPrecisionMinMax<35, -180, 180> Lng_T;
    typedef FixPrecisionMinMax<53, -1000000, 1000000> Fp53_T;

    typedef message_helper::MessageT<
        Fp2_T, Fp10_T, Lat_T, Lng_T, Fp53_T
    > MsgT;

    Fp2_T fp2;
    Fp10_T fp10;
    Lat_T lat;
    Lng_T lng;
    Fp53_T fp53;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<FixedPoints>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<FixedPoints>(); }
    static bitis_helper::BitiaDeserializerHelper<FixedPoints> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<FixedPoints>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[5] = {"fp2", "fp10", "lat", "lng", "fp53"};
    void print(int16_t indent=0) const {
        printf("FixedPoints{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const FixedPoints &other) const {
        return fp2==other.fp2 && fp10==other.fp10 && lat==other.lat && lng==other.lng && fp53==other.fp53;}
    bool operator==(const FixedPoints &other) const { return is_equal(other); }
    bool operator!=(const FixedPoints &other) const { return !is_equal(other); }
};

// ****** Texts *****


struct Texts {
    typedef BitisAString<4> Text_T;
    typedef Binary<4> Data_T;

    typedef message_helper::MessageT<
        Text_T, Data_T
    > MsgT;

    Text_T text;
    Data_T data;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Texts>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Texts>(); }
    static bitis_helper::BitiaDeserializerHelper<Texts> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Texts>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"text", "data"};
    void print(int16_t indent=0) const {
        printf("Texts{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Texts &other) const {
        return text==other.text && data==other.data;}
    bool operator==(const Texts &other) const { return is_equal(other); }
    bool operator!=(const Texts &other) const { return !is_equal(other); }
};

// ****** Leaf *****


struct Leaf {
    typedef IntgralWithGivenBitSize<int16_t, 7> Val_T;
    typedef BitisAString<4> Tag_T;

    typedef message_helper::MessageT<
        Val_T, Tag_T
    > MsgT;

    Val_T val;
    Tag_T tag;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Leaf>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Leaf>(); }
    static bitis_helper::BitiaDeserializerHelper<Leaf> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Leaf>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"val", "tag"};
    void print(int16_t indent=0) const {
        printf("Leaf{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Leaf &other) const {
        return val==other.val && tag==other.tag;}
    bool operator==(const Leaf &other) const { return is_equal(other); }
    bool operator!=(const Leaf &other) const { return !is_equal(other); }
};

// ****** Mid *****


struct Mid {
    typedef IntgralWithGivenBitSize<uint16_t, 7> Id_T;
    typedef Leaf Leaf_T;
    typedef BitisOptional<Leaf> Extra_T;

    typedef message_helper::MessageT<
        Id_T, Leaf_T, Extra_T
    > MsgT;

    Id_T id;
    Leaf_T leaf;
    Extra_T extra;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Mid>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Mid>(); }
    static bitis_helper::BitiaDeserializerHelper<Mid> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Mid>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[3] = {"id", "leaf", "extra"};
    void print(int16_t indent=0) const {
        printf("Mid{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Mid &other) const {
        return id==other.id && leaf==other.leaf && extra==other.extra;}
    bool operator==(const Mid &other) const { return is_equal(other); }
    bool operator!=(const Mid &other) const { return !is_equal(other); }
};

// ****** Nested *****


struct Nested {
    typedef Mid Mid_T;
    typedef BitisBool After_T;

    typedef message_helper::MessageT<
        Mid_T, After_T
    > MsgT;

    Mid_T mid;
    After_T after;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Nested>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Nested>(); }
    static bitis_helper::BitiaDeserializerHelper<Nested> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Nested>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"mid", "after"};
    void print(int16_t indent=0) const {
        printf("Nested{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Nested &other) const {
        return mid==other.mid && after==other.after;}
    bool operator==(const Nested &other) const { return is_equal(other); }
    bool operator!=(const Nested &other) const { return !is_equal(other); }
};

// ****** Containers *****


struct Containers {
    typedef BitisOptional<IntgralWithGivenBitSize<uint16_t, 7>> OptU7_T;
    typedef BitisOptional<Leaf> OptLeaf_T;
    typedef BitisOptional<BitisAString<4>> OptText_T;
    typedef BitisOptional<Color> OptColor_T;
    typedef FixedArray<IntgralWithGivenBitSize<int16_t, 7>,3> FixedInts_T;
    typedef FixedArray<Leaf,2> FixedLeaves_T;
    typedef DynArray<IntgralWithGivenBitSize<uint16_t, 7>,3> DynInts_T;
    typedef DynArray<BitisBool,4> DynFlags_T;
    typedef DynArray<BitisAString<4>,2> DynTexts_T;
    typedef DynArray<Leaf,2> DynLeaves_T;

    typedef message_helper::MessageT<
        OptU7_T, OptLeaf_T, OptText_T, OptColor_T, FixedInts_T, FixedLeaves_T, DynInts_T, DynFlags_T, DynTexts_T, DynLeaves_T
    > MsgT;

    OptU7_T opt_u7;
    OptLeaf_T opt_leaf;
    OptText_T opt_text;
    OptColor_T opt_color;
    FixedInts_T fixed_ints;
    FixedLeaves_T fixed_leaves;
    DynInts_T dyn_ints;
    DynFlags_T dyn_flags;
    DynTexts_T dyn_texts;
    DynLeaves_T dyn_leaves;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Containers>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Containers>(); }
    static bitis_helper::BitiaDeserializerHelper<Containers> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Containers>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[10] = {"opt_u7", "opt_leaf", "opt_text", "opt_color", "fixed_ints", "fixed_leaves", "dyn_ints", "dyn_flags", "dyn_texts", "dyn_leaves"};
    void print(int16_t indent=0) const {
        printf("Containers{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Containers &other) const {
        return opt_u7==other.opt_u7 && opt_leaf==other.opt_leaf && opt_text==other.opt_text && opt_color==other.opt_color && fixed_ints==other.fixed_ints && fixed_leaves==other.fixed_leaves && dyn_ints==other.dyn_ints && dyn_flags==other.dyn_flags && dyn_texts==other.dyn_texts && dyn_leaves==other.dyn_leaves;}
    bool operator==(const Containers &other) const { return is_equal(other); }
    bool operator!=(const Containers &other) const { return !is_equal(other); }
};

// ****** Enums *****


struct Enums {
    typedef Color Color_T;
    typedef Level Level_T;
    typedef DynArray<Color,3> Colors_T;

    typedef message_helper::MessageT<
        Color_T, Level_T, Colors_T
    > MsgT;

    Color_T color;
    Level_T level;
    Colors_T colors;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Enums>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Enums>(); }
    static bitis_helper::BitiaDeserializerHelper<Enums> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Enums>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[3] = {"color", "level", "colors"};
    void print(int16_t indent=0) const {
        printf("Enums{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Enums &other) const {
        return color==other.color && level==other.level && colors==other.colors;}
    bool operator==(const Enums &other) const { return is_equal(other); }
    bool operator!=(const Enums &other) const { return !is_equal(other); }
};

// ****** IntList *****


struct IntList {
    typedef DynArray<IntgralWithGivenBitSize<int16_t, 7>,3> Items_T;

    typedef message_helper::MessageT<
        Items_T
    > MsgT;

    Items_T items;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<IntList>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<IntList>(); }
    static bitis_helper::BitiaDeserializerHelper<IntList> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<IntList>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[1] = {"items"};
    void print(int16_t indent=0) const {
        printf("IntList{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const IntList &other) const {
        return items==other.items;}
    bool operator==(const IntList &other) const { return is_equal(other); }
    bool operator!=(const IntList &other) const { return !is_equal(other); }
};

// ****** OO_Choice_Value *****


struct OO_Choice_Value  {
    struct OO_Flag {
        BITIS_NAME("Flag") typedef BitisBool OOType; };
    struct OO_Num {
        BITIS_NAME("Num") typedef BitisFloatingPoint<double> OOType; };
    struct OO_Text {
        BITIS_NAME("Text") typedef BitisAString<4> OOType; };
    struct OO_Leaf {
        BITIS_NAME("Leaf") typedef Leaf OOType; };
    struct OO_List {
        BITIS_NAME("List") typedef IntList OOType; };
    struct OO_Nothing {
        BITIS_NAME("Nothing") typedef Empty OOType; };
    struct OO_Big {
        BITIS_NAME("Big") typedef DynInteger<int32_t, 32, 7> OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Flag, 
        OO_Num, 
        OO_Text, 
        OO_Leaf, 
        OO_List, 
        OO_Nothing, 
        OO_Big
    >, OO_Flag, 2> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Flag::OOType, OO_Num::OOType, OO_Text::OOType, OO_Leaf::OOType, OO_List::OOType, OO_Nothing::OOType, OO_Big::OOType
    > T_OOValue;
    T_OOValue oo_value;

    OO_Choice_Value() : oo_selector() {}

    template<typename OOT>
    OO_Choice_Value set_oo(typename OOT::OOType v) {
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
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Choice_Value>(); }
    static constexpr std::size_t max_bits() { return oneof_helper::oneof_max_bits<OO_Choice_Value>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Choice_Value> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Choice_Value>(des);
    }

#ifndef BITIS_NO_PRINT
    void print(const int16_t indent=0) const {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }
#else
    void print(const int16_t =0) const { }
#endif

    bool is_equal(const OO_Choice_Value &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Choice_Value &other) const { return is_equal(other); }
    bool operator!=(const OO_Choice_Value &other) const { return !is_equal(other); }
};


// ****** Choice *****


struct Choice {
    typedef OO_Choice_Value Value_T;
    typedef BitisBool Tail_T;

    typedef message_helper::MessageT<
        Value_T, Tail_T
    > MsgT;

    Value_T value;
    Tail_T tail;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Choice>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Choice>(); }
    static bitis_helper::BitiaDeserializerHelper<Choice> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Choice>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"value", "tail"};
    void print(int16_t indent=0) const {
        printf("Choice{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Choice &other) const {
        return value==other.value && tail==other.tail;}
    bool operator==(const Choice &other) const { return is_equal(other); }
    bool operator!=(const Choice &other) const { return !is_equal(other); }
};

// ****** BoundsItem *****


struct BoundsItem {
    typedef IntgralWithGivenBitSize<uint16_t, 4> Val_T;
    typedef BitisOptional<BoundedAString<2, 3>> Label_T;

    typedef message_helper::MessageT<
        Val_T, Label_T
    > MsgT;

    Val_T val;
    Label_T label;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<BoundsItem>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<BoundsItem>(); }
    static bitis_helper::BitiaDeserializerHelper<BoundsItem> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<BoundsItem>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[2] = {"val", "label"};
    void print(int16_t indent=0) const {
        printf("BoundsItem{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const BoundsItem &other) const {
        return val==other.val && label==other.label;}
    bool operator==(const BoundsItem &other) const { return is_equal(other); }
    bool operator!=(const BoundsItem &other) const { return !is_equal(other); }
};

// ****** Bounds *****


struct Bounds {
    typedef BoundedAString<4, 8> Name_T;
    typedef BoundedBinary<3, 4> Blob_T;
    typedef BoundedDynArray<BoundedAString<2, 2>,2,3> Tags_T;
    typedef BitisOptional<BoundedAString<4, 5>> Note_T;
    typedef BoundedDynArray<BoundsItem,3,2> Items_T;

    typedef message_helper::MessageT<
        Name_T, Blob_T, Tags_T, Note_T, Items_T
    > MsgT;

    Name_T name;
    Blob_T blob;
    Tags_T tags;
    Note_T note;
    Items_T items;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Bounds>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Bounds>(); }
    static bitis_helper::BitiaDeserializerHelper<Bounds> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Bounds>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[5] = {"name", "blob", "tags", "note", "items"};
    void print(int16_t indent=0) const {
        printf("Bounds{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Bounds &other) const {
        return name==other.name && blob==other.blob && tags==other.tags && note==other.note && items==other.items;}
    bool operator==(const Bounds &other) const { return is_equal(other); }
    bool operator!=(const Bounds &other) const { return !is_equal(other); }
};

// ****** Everything *****


struct Everything {
    typedef Ints Ints_T;
    typedef DynInts DynInts_T;
    typedef Floats Floats_T;
    typedef FixedPoints FixedPoints_T;
    typedef Texts Texts_T;
    typedef Containers Containers_T;
    typedef Enums Enums_T;
    typedef Choice Choice_T;
    typedef Nested Nested_T;
    typedef Empty Empty_T;
    typedef IntgralWithGivenBitSize<uint16_t, 3> Tail_T;

    typedef message_helper::MessageT<
        Ints_T, DynInts_T, Floats_T, FixedPoints_T, Texts_T, Containers_T, Enums_T, Choice_T, Nested_T, Empty_T, Tail_T
    > MsgT;

    Ints_T ints;
    DynInts_T dyn_ints;
    Floats_T floats;
    FixedPoints_T fixed_points;
    Texts_T texts;
    Containers_T containers;
    Enums_T enums;
    Choice_T choice;
    Nested_T nested;
    Empty_T empty;
    Tail_T tail;

    std::size_t serialize(BitisSerializer &ser) const {
        return message_helper::msg_serialize(this, ser);
    }
    static constexpr std::size_t min_bits() { return message_helper::msg_min_bits<Everything>(); }
    static constexpr std::size_t max_bits() { return message_helper::msg_max_bits<Everything>(); }
    static bitis_helper::BitiaDeserializerHelper<Everything> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<Everything>(des);
    }

#ifndef BITIS_NO_PRINT
    static constexpr const char *msg_attr[11] = {"ints", "dyn_ints", "floats", "fixed_points", "texts", "containers", "enums", "choice", "nested", "empty", "tail"};
    void print(int16_t indent=0) const {
        printf("Everything{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
    }
#else
    void print(int16_t =0) const { }
#endif

    bool is_equal(const Everything &other) const {
        return ints==other.ints && dyn_ints==other.dyn_ints && floats==other.floats && fixed_points==other.fixed_points && texts==other.texts && containers==other.containers && enums==other.enums && choice==other.choice && nested==other.nested && empty==other.empty && tail==other.tail;}
    bool operator==(const Everything &other) const { return is_equal(other); }
    bool operator!=(const Everything &other) const { return !is_equal(other); }
};