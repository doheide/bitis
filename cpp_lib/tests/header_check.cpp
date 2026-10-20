// Compiled with -Wall -Wextra -Wpedantic -Werror -fno-exceptions -fno-rtti (see
// CMakeLists.txt): instantiates the whole runtime API and the generated test messages.
#include "bitis_lib.h"
#include "gen/test_msgs.h"
#include "gen/test_bounds.h"
#include "bitis_lib.h"

template<typename T>
static int roundtrip(const T &v) {
    std::vector<uint8_t> bytes;
    BitisSize size{0, 0};
    if (try_serialize(v, bytes, &size) != BitisError::Ok) { return 1; }
    T out;
    if (try_deserialize(bytes, out, &size) != BitisError::Ok) { return 1; }
    if (try_deserialize(bytes.data(), bytes.size(), out) != BitisError::Ok) { return 1; }
    if (try_deserialize_exact(bytes, out) != BitisError::Ok) { return 1; }
    if (try_deserialize_exact(bytes.data(), bytes.size(), out, &size) != BitisError::Ok) { return 1; }
    if (!(out == v) || out != v) { return 1; }
    if (!(deserialize<T>(serialize(v)) == v)) { return 1; }
    if (deserialize_with_size<T>(bytes).bits != size.bits) { return 1; }
    v.print(-1); printf("\n");
    (void) bitis_min_bits<T>();
    return 0;
}

int main() {
    int errors = 0;
    errors += roundtrip(BitisBool(true));
    errors += roundtrip(IntgralWithGivenBitSize<uint8_t, 8>(200));
    errors += roundtrip(IntgralWithGivenBitSize<int8_t, 8>(-127));
    errors += roundtrip(IntgralWithGivenBitSize<int64_t, 64>(-5));
    errors += roundtrip(IntgralWithGivenBitSize<uint64_t, 64>(5));
    errors += roundtrip(DynInteger<int8_t, 8, 3>(-100));
    errors += roundtrip(DynInteger<uint64_t, 64, 64>(UINT64_MAX));
    errors += roundtrip(BitisFloatingPoint<float>(1.5f));
    errors += roundtrip(FixPrecisionMinMax<2, 0, 1>(0.5));
    errors += roundtrip(FixPrecisionMinMax<53, -1000, 1000>(0.5));
    errors += roundtrip(FixPrecisionMinMax<64, -1000, 1000>(0.5));
    errors += roundtrip(BitisOptional<BitisBool>(BitisBool(true)));
    errors += roundtrip(FixedArray<BitisBool, 3>());
    errors += roundtrip(DynArray<BitisBool, 3>(std::vector<BitisBool>{BitisBool(true)}));
    errors += roundtrip(Binary<4>(std::vector<uint8_t>{1, 2, 3}));
    errors += roundtrip(BitisAString<4>("abc"));
    errors += roundtrip(Color::create_enum<ColorEnum::Cyan>());
    errors += roundtrip(Point());
    errors += roundtrip(Empty());
    errors += roundtrip(AllTypes());
    errors += roundtrip(XL());
    errors += roundtrip(Inner());
    errors += roundtrip(OO_Xl_Val());
    errors += roundtrip(MsgKVMapOO());
    errors += roundtrip(MsgKVMapSimple());
    errors += roundtrip(BoundedBinary<3, 4>({1, 2, 3, 4}));
    errors += roundtrip(BoundedAString<4, 300>("abc"));
    errors += roundtrip(BoundedDynArray<BoundedAString<4, 5>, 2, 3>({BoundedAString<4, 5>("ab")}));
    errors += roundtrip(BoundedDynArray<BitisBool, 1, 65535>());
    errors += roundtrip(Bounded());
    static_assert(BoundedDynArray<BoundedBinary<2, 3>, 2, 4>::max_bits() == 7 + 4 * (4 + 24), "max_bits: dyn(4, 32, 2) = 7");

    AllTypes a;
    a.payload.set_oo<OO_AllTypes_Payload::OO_Text>(OO_AllTypes_Payload::OO_Text::OOType("text"));
    errors += roundtrip(a);
    a.payload.set_oo<OO_AllTypes_Payload::OO_Pt>(Point());
    errors += roundtrip(a);
    const AllTypes &ca = a;
    if (!ca.payload.get_oo<OO_AllTypes_Payload::OO_Pt>() || !ca.payload.is_oo_value<OO_AllTypes_Payload::OO_Pt>()) { errors++; }

    BitisDeserializer des(std::vector<uint8_t>{0x01});
    BitisDeserializer des2 = des;
    errors += des2.decode_data_uint8(1).data == 1 ? 0 : 1;
    uint16_t raw = 0;
    des.decode_data(raw, 8);
    errors += (raw == 1) ? 0 : 1;
    BitisSerializer ser;
    const uint32_t v = 0x12345;
    ser.add_data<uint32_t, 20>(&v);
    ser.add_data_bits(&v, 4);
    errors += (ser.finalize().bits == 24) ? 0 : 1;
    errors += (calc_bitmask(64) == UINT64_MAX && calc_bitmask(3) == 7) ? 0 : 1;
    printf("%s\n", bitis_error_name(BitisError::TrailingData));

    printf("header_check: %s\n", errors ? "FAILED" : "OK");
    return errors;
}
