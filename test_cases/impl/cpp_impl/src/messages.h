#include "bitis_lib.h"
#include <optional>

//#define EXPECTED_BITIS_VERSION "0.10.2"
//#if EXPECTED_BITIS_VERSION != BITIS_CPP_LIB_VERSION
//#error "Unexpected bitis library version"
//#endif



// ****** MsgSimpleBaseOneInt *****


struct MsgSimpleBaseOneInt {
    typedef IntgralWithGivenBitSize<uint16_t, 11> Param1_T;

    typedef message_helper::MessageT<
        Param1_T
    > MsgT;

    Param1_T param_1;
    const char * str_param_1 = "param_1";
    const char *msg_attr[1] = {str_param_1};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgSimpleBaseOneInt> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgSimpleBaseOneInt>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgSimpleBaseOneInt{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgSimpleBaseOneInt &other) const {
        return param_1==other.param_1;}
    bool operator==(const MsgSimpleBaseOneInt &other) const { return is_equal(other); }
    bool operator!=(const MsgSimpleBaseOneInt &other) const { return !is_equal(other); }
};
//const char *MsgSimpleBaseOneInt::msg_attr[] = {"param_1"};

// ****** OO_MsgOoSimpleBase_Value *****


struct OO_MsgOoSimpleBase_Value  {
    struct OO_Int {
        static constexpr auto name = "Int"; typedef IntgralWithGivenBitSize<uint16_t, 8> OOType; };
    struct OO_Number {
        static constexpr auto name = "Number"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_TrueFalse {
        static constexpr auto name = "TrueFalse"; typedef BitisBool OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Int, 
        OO_Number, 
        OO_TrueFalse
    >, OO_TrueFalse, 4> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Int::OOType, OO_Number::OOType, OO_TrueFalse::OOType
    > T_OOValue;
    T_OOValue oo_value;
    //uint8_t *oo_value_ptr;

    OO_MsgOoSimpleBase_Value() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_MsgOoSimpleBase_Value() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_MsgOoSimpleBase_Value set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(v);
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

    std::size_t serialize(BitisSerializer &ser) {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<OO_MsgOoSimpleBase_Value> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_MsgOoSimpleBase_Value>(des);
    }

    void print(const int16_t indent=0) {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_MsgOoSimpleBase_Value &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_MsgOoSimpleBase_Value &other) const { return is_equal(other); }
    bool operator!=(const OO_MsgOoSimpleBase_Value &other) const { return !is_equal(other); }
};


// ****** OO_MsgOoNestedBase_Value *****


struct OO_MsgOoNestedBase_Value  {
    struct OO_Inner {
        static constexpr auto name = "Inner"; typedef MsgSimpleBaseOneInt OOType; };
    struct OO_Number {
        static constexpr auto name = "Number"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_TrueFalse {
        static constexpr auto name = "TrueFalse"; typedef BitisBool OOType; };

    typedef BitisEnum<bitis_helper::Collector<
        OO_Inner, 
        OO_Number, 
        OO_TrueFalse
    >, OO_Inner, 4> T_OOEnum;
    T_OOEnum oo_selector;

    typedef oneof_helper::UnionT<  // UnionDynSizeT
        OO_Inner::OOType, OO_Number::OOType, OO_TrueFalse::OOType
    > T_OOValue;
    T_OOValue oo_value;
    //uint8_t *oo_value_ptr;

    OO_MsgOoNestedBase_Value() : oo_selector() /*, oo_value_ptr(nullptr)*/ {}
    //~OO_MsgOoNestedBase_Value() { delete [] oo_value_ptr; }

    template<typename OOT>
    OO_MsgOoNestedBase_Value set_oo(typename OOT::OOType v) {
        static_assert(oneof_helper::ContainsType<OOT, T_OOEnum::EnumCollector>::value);
        oo_selector.set_enum<OOT>();
        oo_value.set(v);
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

    std::size_t serialize(BitisSerializer &ser) {
        return oneof_helper::oneof_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<OO_MsgOoNestedBase_Value> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_MsgOoNestedBase_Value>(des);
    }

    void print(const int16_t indent=0) {
        printf("Oneof = ");
        oneof_helper::oneof_print(this, (indent>=0) ? indent + 2 : indent);
    }

    bool is_equal(const OO_MsgOoNestedBase_Value &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_MsgOoNestedBase_Value &other) const { return is_equal(other); }
    bool operator!=(const OO_MsgOoNestedBase_Value &other) const { return !is_equal(other); }
};


// ****** MsgOOSimpleBase *****


struct MsgOOSimpleBase {
    typedef IntgralWithGivenBitSize<uint16_t, 8> Id_T;
    typedef OO_MsgOoSimpleBase_Value Value_T;

    typedef message_helper::MessageT<
        Id_T, Value_T
    > MsgT;

    Id_T id;
    Value_T value;
    const char * str_id = "id";
    const char * str_value = "value";
    const char *msg_attr[2] = {str_id, str_value};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgOOSimpleBase> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgOOSimpleBase>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgOOSimpleBase{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgOOSimpleBase &other) const {
        return id==other.id && value==other.value;}
    bool operator==(const MsgOOSimpleBase &other) const { return is_equal(other); }
    bool operator!=(const MsgOOSimpleBase &other) const { return !is_equal(other); }
};
//const char *MsgOOSimpleBase::msg_attr[] = {"id", "value"};

// ****** MsgOONestedBase *****


struct MsgOONestedBase {
    typedef IntgralWithGivenBitSize<uint16_t, 8> Id_T;
    typedef OO_MsgOoNestedBase_Value Value_T;

    typedef message_helper::MessageT<
        Id_T, Value_T
    > MsgT;

    Id_T id;
    Value_T value;
    const char * str_id = "id";
    const char * str_value = "value";
    const char *msg_attr[2] = {str_id, str_value};

    //static const char *msg_attr[];

    std::size_t serialize(BitisSerializer &ser) {
        return message_helper::msg_serialize(this, ser);
    }
    static bitis_helper::BitiaDeserializerHelper<MsgOONestedBase> deserialize(BitisDeserializer &des) {
        return message_helper::msg_deserialize<MsgOONestedBase>(des);
    }

    void print(int16_t indent=0) {
        printf("MsgOONestedBase{ ");
        if (indent>=0) printf("\n");
        message_helper::msg_print(this, (indent>=0) ? (2 + indent) : indent, msg_attr);
        print_indent(indent); printf("}");
        // if (indent>=0) printf("\n");
    }

    bool is_equal(const MsgOONestedBase &other) const {
        return id==other.id && value==other.value;}
    bool operator==(const MsgOONestedBase &other) const { return is_equal(other); }
    bool operator!=(const MsgOONestedBase &other) const { return !is_equal(other); }
};
//const char *MsgOONestedBase::msg_attr[] = {"id", "value"};