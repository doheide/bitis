// B3: oneof storage (construction / destruction / copy / move / alignment)
#include <gtest/gtest.h>
#include <cstring>

#include "bitis_lib.h"
#include "test_helpers.h"

using namespace bitis_test;

namespace test_oneof_storage {

// bitis value type that counts its live instances
struct Tracked {
    static int live;
    static int constructed;
    IntgralWithGivenBitSize<uint16_t, 8> v;
    std::vector<uint8_t> heap;   // forces real heap ownership

    Tracked() : v(), heap(16, 0x11) { live++; constructed++; }
    explicit Tracked(uint16_t x) : v(x), heap(16, static_cast<uint8_t>(x)) { live++; constructed++; }
    Tracked(const Tracked &o) : v(o.v), heap(o.heap) { live++; constructed++; }
    Tracked(Tracked &&o) noexcept : v(o.v), heap(std::move(o.heap)) { live++; constructed++; }
    Tracked &operator=(const Tracked &o) = default;
    Tracked &operator=(Tracked &&o) noexcept = default;
    ~Tracked() { live--; }

    static constexpr std::size_t min_bits() { return 8; }
    std::size_t serialize(BitisSerializer &ser) const { return v.serialize(ser); }
    static bitis_helper::BitiaDeserializerHelper<Tracked> deserialize(BitisDeserializer &des) {
        auto r = IntgralWithGivenBitSize<uint16_t, 8>::deserialize(des);
        return bitis_helper::BitiaDeserializerHelper<Tracked>{r.bits, Tracked(r.data.value)};
    }
    void print(int16_t indent=0) const { v.print(indent); }
    bool is_equal(const Tracked &o) const { return v == o.v; }
    bool operator==(const Tracked &o) const { return is_equal(o); }
    bool operator!=(const Tracked &o) const { return !is_equal(o); }
};
int Tracked::live = 0;
int Tracked::constructed = 0;

// hand written oneof in the shape of the generated code (templates/data_objects.cpp.jinja)
struct OO_Test {
    struct OO_Tr { static constexpr auto name = "Tr"; typedef Tracked OOType; };
    struct OO_Str { static constexpr auto name = "Str"; typedef BitisAString<4> OOType; };
    struct OO_Dbl { static constexpr auto name = "Dbl"; typedef BitisFloatingPoint<double> OOType; };
    struct OO_Flag { static constexpr auto name = "Flag"; typedef BitisBool OOType; };

    typedef BitisEnum<bitis_helper::Collector<OO_Tr, OO_Str, OO_Dbl, OO_Flag>, OO_Flag, 2> T_OOEnum;
    T_OOEnum oo_selector;
    typedef oneof_helper::UnionT<OO_Tr::OOType, OO_Str::OOType, OO_Dbl::OOType, OO_Flag::OOType> T_OOValue;
    T_OOValue oo_value;

    OO_Test() : oo_selector() {}

    template<typename OOT>
    OO_Test set_oo(typename OOT::OOType v) {
        oo_selector.set_enum<OOT>();
        oo_value.set(std::move(v));
        return *this;
    }
    template<typename OOT>
    typename OOT::OOType *get_oo() const {
        if (oo_selector.is_enum<OOT>()) return oo_value.get<typename OOT::OOType>();
        return nullptr;
    }
    std::size_t serialize(BitisSerializer &ser) const { return oneof_helper::oneof_serialize(this, ser); }
    static constexpr std::size_t min_bits() { return oneof_helper::oneof_min_bits<OO_Test>(); }
    static bitis_helper::BitiaDeserializerHelper<OO_Test> deserialize(BitisDeserializer &des) {
        return oneof_helper::oneof_deserialize<OO_Test>(des);
    }
    void print(const int16_t indent=0) const { oneof_helper::oneof_print(this, indent); }
    bool is_equal(const OO_Test &other) const {
        if (oo_selector != other.oo_selector) return false;
        return oneof_helper::oneof_is_equal(this, &other);
    }
    bool operator==(const OO_Test &other) const { return is_equal(other); }
    bool operator!=(const OO_Test &other) const { return !is_equal(other); }
};

class BITIS_OneOfStorage : public ::testing::Test {
protected:
    void SetUp() override { Tracked::live = 0; }
    void TearDown() override { EXPECT_EQ(Tracked::live, 0); }
};

TEST_F(BITIS_OneOfStorage, ConstructDestroy) {
    {
        OO_Test a;
        a.set_oo<OO_Test::OO_Tr>(Tracked(5));
        EXPECT_EQ(Tracked::live, 1);
        EXPECT_EQ(a.get_oo<OO_Test::OO_Tr>()->v.value, 5);
        EXPECT_EQ(a.get_oo<OO_Test::OO_Str>(), nullptr);
    }
    EXPECT_EQ(Tracked::live, 0);
}

TEST_F(BITIS_OneOfStorage, SwitchVariants) {
    OO_Test a;
    a.set_oo<OO_Test::OO_Tr>(Tracked(1));
    a.set_oo<OO_Test::OO_Str>(BitisAString<4>("a heap string that is long enough"));
    EXPECT_EQ(Tracked::live, 0);   // Tracked destroyed when switching
    EXPECT_STREQ(a.get_oo<OO_Test::OO_Str>()->get(), "a heap string that is long enough");
    a.set_oo<OO_Test::OO_Dbl>(BitisFloatingPoint<double>(1.5));
    a.set_oo<OO_Test::OO_Tr>(Tracked(2));
    a.set_oo<OO_Test::OO_Tr>(Tracked(3));   // same variant: assignment
    EXPECT_EQ(Tracked::live, 1);
    EXPECT_EQ(a.get_oo<OO_Test::OO_Tr>()->v.value, 3);
    a.oo_value.reset();
    EXPECT_FALSE(a.oo_value.has_value());
    EXPECT_EQ(Tracked::live, 0);
}

TEST_F(BITIS_OneOfStorage, CopyIsDeep) {
    OO_Test a;
    a.set_oo<OO_Test::OO_Str>(BitisAString<4>("AAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
    OO_Test b = a;
    b.set_oo<OO_Test::OO_Str>(BitisAString<4>("BBBB"));
    EXPECT_STREQ(a.get_oo<OO_Test::OO_Str>()->get(), "AAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    EXPECT_STREQ(b.get_oo<OO_Test::OO_Str>()->get(), "BBBB");
    EXPECT_NE(a.get_oo<OO_Test::OO_Str>()->get(), b.get_oo<OO_Test::OO_Str>()->get());

    a.set_oo<OO_Test::OO_Tr>(Tracked(9));
    OO_Test c(a);
    EXPECT_EQ(Tracked::live, 2);
    EXPECT_NE(c.get_oo<OO_Test::OO_Tr>()->heap.data(), a.get_oo<OO_Test::OO_Tr>()->heap.data());
    EXPECT_TRUE(c == a);
    c = b;                      // Tracked -> string
    EXPECT_EQ(Tracked::live, 1);
    EXPECT_TRUE(c == b);
    c = a;                      // string -> Tracked
    EXPECT_EQ(Tracked::live, 2);
    const OO_Test &self = c;
    c = self;                   // self assignment
    EXPECT_EQ(Tracked::live, 2);
    EXPECT_EQ(c.get_oo<OO_Test::OO_Tr>()->v.value, 9);
}

TEST_F(BITIS_OneOfStorage, Move) {
    OO_Test a;
    a.set_oo<OO_Test::OO_Tr>(Tracked(7));
    const uint8_t *heap = a.get_oo<OO_Test::OO_Tr>()->heap.data();
    OO_Test b(std::move(a));
    EXPECT_EQ(b.get_oo<OO_Test::OO_Tr>()->heap.data(), heap);   // moved, not copied
    OO_Test c;
    c.set_oo<OO_Test::OO_Str>(BitisAString<4>("x"));
    c = std::move(b);
    EXPECT_EQ(c.get_oo<OO_Test::OO_Tr>()->v.value, 7);
    EXPECT_EQ(c.get_oo<OO_Test::OO_Tr>()->heap.data(), heap);
    // a and b hold moved-from Tracked objects that are still destroyed correctly
}

TEST_F(BITIS_OneOfStorage, SetFromOwnValue) {
    OO_Test a;
    a.set_oo<OO_Test::OO_Tr>(Tracked(4));
    a.oo_value.set(*a.oo_value.get<Tracked>());   // aliasing the stored value
    EXPECT_EQ(a.get_oo<OO_Test::OO_Tr>()->v.value, 4);
    a.oo_value.emplace<Tracked>(*a.oo_value.get<Tracked>());
    EXPECT_EQ(a.get_oo<OO_Test::OO_Tr>()->v.value, 4);
    EXPECT_EQ(Tracked::live, 1);
}

TEST_F(BITIS_OneOfStorage, DefaultOneofHasDefaultValue) {
    OO_Test a;   // default variant Flag, nothing stored yet
    ASSERT_NE(a.get_oo<OO_Test::OO_Flag>(), nullptr);
    EXPECT_FALSE(a.get_oo<OO_Test::OO_Flag>()->value);
    EXPECT_EQ(a.get_oo<OO_Test::OO_Tr>(), nullptr);
    // selector changed by hand: get_oo() provides a default constructed value
    a.oo_selector.set_enum<OO_Test::OO_Tr>();
    ASSERT_NE(a.get_oo<OO_Test::OO_Tr>(), nullptr);
    EXPECT_EQ(a.get_oo<OO_Test::OO_Tr>()->v.value, 0);
    EXPECT_EQ(Tracked::live, 1);
    // a default oneof serializes its default variant
    EXPECT_EQ(serialize(OO_Test()), std::vector<uint8_t>{0x07 | 0x00});   // id 3 (1,11,0) + false
}

TEST_F(BITIS_OneOfStorage, Alignment) {
    struct Packed { uint8_t pad; OO_Test oo; };
    Packed p;
    p.oo.set_oo<OO_Test::OO_Dbl>(BitisFloatingPoint<double>(2.0));
    const auto addr = reinterpret_cast<std::uintptr_t>(p.oo.get_oo<OO_Test::OO_Dbl>());
    EXPECT_EQ(addr % alignof(double), 0u);
    EXPECT_GE(alignof(OO_Test::T_OOValue), alignof(double));
    EXPECT_GE(alignof(OO_Test::T_OOValue), alignof(std::vector<uint8_t>));
}

TEST_F(BITIS_OneOfStorage, SerializeDeserialize) {
    OO_Test a;
    a.set_oo<OO_Test::OO_Tr>(Tracked(200));
    auto bytes = serialize(a);
    OO_Test d;
    EXPECT_EQ(try_deserialize(bytes, d), BitisError::Ok);
    EXPECT_TRUE(d == a);
    EXPECT_EQ(Tracked::live, 2);
    EXPECT_EQ(OO_Test::min_bits(), 2u);   // 1 + bool
    // unknown index 4..
    auto bad = Bits().add(DynInteger<uint32_t, 32, 2>(4)).add(0, 16).bytes();
    EXPECT_EQ(try_deserialize(bad, d), BitisError::InvalidDiscriminant);
    EXPECT_TRUE(d == a);
}

TEST_F(BITIS_OneOfStorage, VectorOfMessagesWithOneofs) {
    // reallocation of a vector moves/copies messages that contain oneofs
    std::vector<MsgKVOO> v;
    for (int i = 0; i < 100; i++) {
        MsgKVOO m; m.key = MsgKVOO::Key_T("k");
        if (i % 2) {
            m.value.set_oo<OO_MsgKvoo_Value::OO_StrVal>(
                OO_MsgKvoo_Value::OO_StrVal::OOType("value string number with some length"));
        } else {
            m.value.set_oo<OO_MsgKvoo_Value::OO_NumVal>(OO_MsgKvoo_Value::OO_NumVal::OOType(i));
        }
        v.push_back(m);
    }
    v.erase(v.begin() + 3, v.begin() + 50);
    EXPECT_EQ(v.size(), 53u);
    EXPECT_STREQ(v[4].value.get_oo<OO_MsgKvoo_Value::OO_StrVal>()->get(), "value string number with some length");
    MsgKVMapOO map; map.entries.values = v;
    auto bytes = serialize(map);
    MsgKVMapOO d;
    ASSERT_EQ(try_deserialize(bytes, d), BitisError::Ok);
    EXPECT_TRUE(d == map);
}

// the scenarios of the original review (kv/oo.cpp): leak, alias, use after free, switch
TEST_F(BITIS_OneOfStorage, ReviewScenarios) {
    typedef OO_MsgKvoo_Value OV;
    {   // leak
        MsgKVOO m; m.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("a string value that lives on the heap"));
        auto b = serialize(m);
        auto d = deserialize<MsgKVOO>(b);
        EXPECT_TRUE(d == m);
    }
    {   // alias
        MsgKVOO a; a.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("AAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
        MsgKVOO b = a;
        b.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("BBBB"));
        auto pa = a.value.get_oo<OV::OO_StrVal>(); auto pb = b.value.get_oo<OV::OO_StrVal>();
        EXPECT_EQ(pa->size(), 28u);
        EXPECT_EQ(pb->size(), 4u);
        EXPECT_NE(pa->get_u8_vec()->data(), pb->get_u8_vec()->data());
        EXPECT_EQ(pa->get()[0], 'A');
    }
    {   // use after free
        MsgKVOO a; a.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("AA"));
        MsgKVOO b = a;
        b.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("a much longer string that forces reallocation"));
        EXPECT_STREQ(a.value.get_oo<OV::OO_StrVal>()->get(), "AA");
    }
    {   // switch
        MsgKVOO a; a.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("some heap string ..........."));
        a.value.set_oo<OV::OO_NumVal>(OV::OO_NumVal::OOType(1.5));
        a.value.set_oo<OV::OO_StrVal>(OV::OO_StrVal::OOType("x"));
        EXPECT_STREQ(a.value.get_oo<OV::OO_StrVal>()->get(), "x");
    }
    {   // chained set_oo on temporaries (firmware pattern)
        XL m;
        m.val = OO_Xl_Val().set_oo<OO_Xl_Val::OO_Inner>(Inner{Inner::Va_T(3), BitisBool(true)});
        EXPECT_EQ(m.val.get_oo<OO_Xl_Val::OO_Inner>()->va.value, 3);
    }
}

}
