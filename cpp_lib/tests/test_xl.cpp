// Cross-language check: bytes produced by the Rust reference runtime (bitis 0.10.5,
// src/lib_impl/berde.rs) for the XL test vectors (schema in test_msgs.bitis, values in
// test_helpers.h). Vector 4 contains the fixed point value that differed before (B5).
#include <gtest/gtest.h>

#include "bitis_lib.h"
#include "test_helpers.h"

using namespace bitis_test;

namespace test_xl {

static const char *RUST_XL_BYTES[] = {
    "0000000000000000000000000000000000FEFF7F000006",
    "DDFEFFFFFFFFFF6FBA49AC8BD14244B51F9200846200FCFB7DC0A5E007D4CD8256C6C6F636000000000000088001",
    "0004000200FF7F0000C03F0000000000000040FFFD9F211E8A01",
    "00000000407C00000000000000000000000000E0FF010018E74ACD771D",
    "0000000000842D020000000000000020008208000000C601",
    "0000000000000000000000000000000000FEFF7F0000261441206C6F6E672D69736820737472696E67207769746820333220636861727321C7FFFFFFFF03",
};

TEST(BITIS_CrossLanguage, EncodeMatchesRust) {
    const auto vectors = xl_vectors();
    ASSERT_EQ(vectors.size(), sizeof(RUST_XL_BYTES) / sizeof(RUST_XL_BYTES[0]));
    for (std::size_t i = 0; i < vectors.size(); i++) {
        std::vector<uint8_t> out;
        ASSERT_EQ(try_serialize(vectors[i], out), BitisError::Ok) << "vector " << i;
        EXPECT_EQ(hex(out), RUST_XL_BYTES[i]) << "vector " << i;
    }
}

TEST(BITIS_CrossLanguage, DecodeRustBytes) {
    const auto vectors = xl_vectors();
    for (std::size_t i = 0; i < vectors.size(); i++) {
        const auto bytes = unhex(RUST_XL_BYTES[i]);
        XL m;
        BitisSize size{0, 0};
        ASSERT_EQ(try_deserialize_exact(bytes, m, &size), BitisError::Ok) << "vector " << i;
        EXPECT_EQ(size.bytes, bytes.size());
        EXPECT_EQ(hex(serialize(m)), RUST_XL_BYTES[i]) << "vector " << i;
        EXPECT_TRUE(m == vectors[i]) << "vector " << i;
    }
}

}
