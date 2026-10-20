// helpers shared by the bitis C++ runtime tests
#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <cstdio>

#include "gen/test_msgs.h"

namespace bitis_test {
    inline std::string hex(const std::vector<uint8_t> &v) {
        std::string s; char b[3];
        for (auto c : v) { snprintf(b, sizeof(b), "%02X", c); s += b; }
        return s;
    }
    inline std::vector<uint8_t> unhex(const std::string &s) {
        std::vector<uint8_t> v;
        for (std::size_t i = 0; i + 1 < s.size(); i += 2) {
            v.push_back(static_cast<uint8_t>(std::stoul(s.substr(i, 2), nullptr, 16)));
        }
        return v;
    }

    // builds raw bit streams (LSB first) for malformed input
    struct Bits {
        BitisSerializer ser;
        Bits &add(uint64_t v, uint8_t num_bits) { ser.write_bits(v, num_bits); return *this; }
        template<typename T> Bits &add(const T &bitis_value) { bitis_value.serialize(ser); return *this; }
        std::vector<uint8_t> bytes() { ser.finalize(); return ser.data_cache; }
    };

    // deterministic PRNG (xorshift64*)
    struct Rng {
        uint64_t s;
        explicit Rng(uint64_t seed) : s(seed ? seed : 0x9E3779B97F4A7C15ull) {}
        uint64_t next() { s ^= s >> 12; s ^= s << 25; s ^= s >> 27; return s * 2685821657736338717ull; }
        uint32_t below(uint32_t n) { return static_cast<uint32_t>(next() % n); }
    };

    typedef OO_Xl_Val OV;
    // the cross-language test vectors (same values as the Rust side of test_xl.cpp)
    inline std::vector<XL> xl_vectors() {
        std::vector<XL> v;
        v.push_back(XL());
        { XL m; m.u3=XL::U3_T(5); m.i5=XL::I5_T(-13); m.i15=XL::I15_T(-32767); m.u13=XL::U13_T(8191);
          m.ud=XL::Ud_T(15); m.sd=XL::Sd_T(-4095); m.fl=XL::Fl_T(-1.5e-3f); m.db=XL::Db_T(3.141592653589793);
          m.fp=XL::Fp_T(0.7); m.fp2=XL::Fp2_T(-1999.5); m.oi=XL::Oi_T(IntgralWithGivenBitSize<int16_t,7>(-63));
          Inner in; in.va=Inner::Va_T(-15); in.vb=BitisBool(true); m.oinner=XL::Oinner_T(in);
          m.fa=XL::Fa_T({IntgralWithGivenBitSize<int16_t,4>(-7),IntgralWithGivenBitSize<int16_t,4>(0),IntgralWithGivenBitSize<int16_t,4>(7)});
          m.da=XL::Da_T({IntgralWithGivenBitSize<uint16_t,6>(1),IntgralWithGivenBitSize<uint16_t,6>(63),IntgralWithGivenBitSize<uint16_t,6>(0),IntgralWithGivenBitSize<uint16_t,6>(42)});
          m.col=Color::create_enum<ColorEnum::Yellow>(); m.str=XL::Str_T("hello");
          m.val=OV().set_oo<OV::OO_Num>(OV::OO_Num::OOType(-2.5)); v.push_back(m); }
        { XL m; m.i15=XL::I15_T(1); m.u13=XL::U13_T(1); m.sd=XL::Sd_T(4095); m.fl=XL::Fl_T(INFINITY); m.db=XL::Db_T(-0.0);
          m.fp=XL::Fp_T(3.0); m.fp2=XL::Fp2_T(18000.0);
          m.fa=XL::Fa_T({IntgralWithGivenBitSize<int16_t,4>(-1),IntgralWithGivenBitSize<int16_t,4>(1),IntgralWithGivenBitSize<int16_t,4>(-7)});
          m.col=Color::create_enum<ColorEnum::Red>(); m.str=XL::Str_T("");
          Inner in; in.va=Inner::Va_T(3); in.vb=BitisBool(false);
          m.val=OV().set_oo<OV::OO_Inner>(in); v.push_back(m); }
        { XL m; m.fp=XL::Fp_T(-2.5); m.fp2=XL::Fp2_T(20000.0); m.val=OV().set_oo<OV::OO_Big>(OV::OO_Big::OOType(-123456789));
          m.sd=XL::Sd_T(-1); m.ud=XL::Ud_T(8); v.push_back(m); }
        { XL m; m.fp=XL::Fp_T(-1.689030362389814); m.fp2=XL::Fp2_T(-2000.0); m.val=OV().set_oo<OV::OO_Big>(OV::OO_Big::OOType(0));
          m.fl=XL::Fl_T(1.0e-40f); m.db=XL::Db_T(DBL_MIN); v.push_back(m); }
        { XL m; m.val=OV().set_oo<OV::OO_Big>(OV::OO_Big::OOType(2147483647)); m.str=XL::Str_T("A long-ish string with 32 chars!"); v.push_back(m); }
        return v;
    }

    inline AllTypes all_types_sample(int variant) {
        AllTypes m;
        m.flag = true;
        m.u3 = AllTypes::U3_T(6);
        m.i5 = AllTypes::I5_T(-31);
        m.i16 = AllTypes::I16_T(-32767);
        m.ud = AllTypes::Ud_T(4000000000u);
        m.sd = AllTypes::Sd_T(-1234);
        m.fl = AllTypes::Fl_T(1.25f);
        m.db = AllTypes::Db_T(-2.5e100);
        m.fp = AllTypes::Fp_T(1.234);
        m.ou = AllTypes::Ou_T(AllTypes::Ou_T::ValT(127));
        m.fa = AllTypes::Fa_T({AllTypes::Fa_T::ValT(-7), AllTypes::Fa_T::ValT(7)});
        m.da = AllTypes::Da_T({AllTypes::Da_T::ValT(1), AllTypes::Da_T::ValT(2), AllTypes::Da_T::ValT(63)});
        m.mode = Mode::create_enum<ModeEnum::Standby>();
        m.name = AllTypes::Name_T("bitis ascii");
        Point p; p.px = Point::Px_T(-32767); p.py = Point::Py_T(INT32_MAX); p.pz = Point::Pz_T(-INT64_MAX);
        p.pw = Point::Pw_T(UINT64_MAX);
        m.points.values.push_back(p);
        m.points.values.push_back(Point());
        typedef OO_AllTypes_Payload PL;
        switch (variant % 5) {
            case 0: m.payload.set_oo<PL::OO_Flag>(PL::OO_Flag::OOType(true)); break;
            case 1: m.payload.set_oo<PL::OO_Text>(PL::OO_Text::OOType("payload text")); break;
            case 2: m.payload.set_oo<PL::OO_Num>(PL::OO_Num::OOType(0.125)); break;
            case 3: m.payload.set_oo<PL::OO_Pt>(p); break;
            default: m.payload.set_oo<PL::OO_Nothing>(Empty()); break;
        }
        return m;
    }

    inline MsgKVMapOO kv_sample() {
        typedef OO_MsgKvoo_Value KV;
        MsgKVMapOO m;
        MsgKVOO e;
        e.key = MsgKVOO::Key_T("name"); e.value.set_oo<KV::OO_StrVal>(KV::OO_StrVal::OOType("a string value on the heap"));
        m.entries.values.push_back(e);
        e.key = MsgKVOO::Key_T("pi"); e.value.set_oo<KV::OO_NumVal>(KV::OO_NumVal::OOType(3.14159));
        m.entries.values.push_back(e);
        e.key = MsgKVOO::Key_T("on"); e.value.set_oo<KV::OO_BoolVal>(KV::OO_BoolVal::OOType(true));
        m.entries.values.push_back(e);
        return m;
    }
}
