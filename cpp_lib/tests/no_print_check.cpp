// Compiled with BITIS_NO_PRINT (and -Wall -Wextra -Wpedantic -Werror -fno-exceptions
// -fno-rtti, see CMakeLists.txt): every print() is callable but empty, no name tables
// exist and the object file does not reference printf (checked by the test
// bitis_no_print_symbols with nm). This file must not print anything itself.
#define BITIS_NO_PRINT
#include "bitis_lib.h"
#include "gen/test_msgs.h"

namespace {
    template<typename T, typename = void> struct HasMsgAttr : std::false_type { };
    template<typename T> struct HasMsgAttr<T, std::void_t<decltype(T::msg_attr)>> : std::true_type { };
    template<typename T, typename = void> struct HasName : std::false_type { };
    template<typename T> struct HasName<T, std::void_t<decltype(T::name)>> : std::true_type { };

    static_assert(!HasMsgAttr<AllTypes>::value && !HasMsgAttr<Empty>::value, "no attribute name table");
    static_assert(!HasName<ModeEnum::Standby>::value && !HasName<ColorEnum::Red>::value, "no enum value names");
    static_assert(!HasName<OO_Xl_Val::OO_Num>::value, "no oneof element names");
    static_assert(sizeof(Inner) == sizeof(Inner::MsgT), "messages hold only their attributes");

    template<typename T>
    int check(const T &v) {
        v.print();
        v.print(-1);
        v.print(4);
        // (de)serialization is not affected
        std::vector<uint8_t> bytes;
        if (try_serialize(v, bytes) != BitisError::Ok) { return 1; }
        T back;
        if (try_deserialize_exact(bytes, back) != BitisError::Ok) { return 1; }
        return (back == v) ? 0 : 1;
    }
}

int main() {
    int errors = 0;
    print_indent(4);
    errors += check(BitisBool(true));
    errors += check(IntgralWithGivenBitSize<int8_t, 7>(-3));
    errors += check(DynInteger<uint32_t, 32, 4>(77));
    errors += check(BitisFloatingPoint<double>(1.5));
    errors += check(FixPrecisionMinMax<10, -2, 3>(0.5));
    errors += check(BitisOptional<BitisBool>(BitisBool(true)));
    errors += check(FixedArray<BitisBool, 2>());
    errors += check(DynArray<BitisBool, 3>(std::vector<BitisBool>{BitisBool(true)}));
    errors += check(Binary<4>(std::vector<uint8_t>{1, 2}));
    errors += check(BitisAString<4>("abc"));
    errors += check(Color::create_enum<ColorEnum::Cyan>());
    errors += check(Empty());
    errors += check(Point());
    errors += check(AllTypes());
    errors += check(XL());
    OO_Xl_Val o;
    o.set_oo<OO_Xl_Val::OO_Num>(OO_Xl_Val::OO_Num::OOType(2.5));
    errors += check(o);
    return errors;
}
