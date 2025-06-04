//
// Created by dheide on 12.05.25.
//
#include "messages.h"
#include <iostream>
#include <fstream>
#include <libintl.h>

template <typename T>
void write_or_test(const char *fn_name, T &msg, const char *arg) {
    auto repl_str = std::string(".cpp.");

    if(arg==nullptr) {
        auto bin_msg = serialize(msg);

        std::ofstream outfile{fn_name, std::ios::binary};
        outfile.write(reinterpret_cast<const char *>(bin_msg.data()), bin_msg.size());
        outfile.close();
    }
    else {
        auto fn_mod = std::string(fn_name);
        auto new_str = "." + std::string(arg) + ".";
        fn_mod.replace(fn_mod.find(repl_str), repl_str.length(), new_str);

        printf("loading: %s\n", fn_mod.c_str());

        std::ifstream infile(fn_mod, std::ifstream::binary);

        infile.seekg(0, infile.end);     //N is the total number of doubles
        auto file_size = infile.tellg();
        infile.seekg(0, infile.beg);

        std::vector<uint8_t> bin_data(file_size);
        infile.read(reinterpret_cast<char *>(bin_data.data()), file_size);
        infile.close();

        auto bin_msg = deserialize<T>(bin_data);
        printf("deserialized:\n"); bin_msg.print();printf("\n");
        if (msg == bin_msg) {
            printf("* OK\n");
        }
        else {
            printf("* FAILED\n");
        }
    }
}

int main(int argc, char *argv[]){
    char *arg = nullptr;
    if (argc > 1) arg = argv[1];

    {
        auto msg = MsgSimpleBaseOneInt{
            .param_1 = MsgSimpleBaseOneInt::Param1_T(1122),
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_simple_one_int.cpp.dat";
        write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgSimpleBaseThreeInt{
            .param_1 = MsgSimpleBaseThreeInt::Param1_T(1122),
            .param_2 = MsgSimpleBaseThreeInt::Param2_T(3),
            .param_3 = MsgSimpleBaseThreeInt::Param3_T(3),
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_simple_three_int.cpp.dat";
        write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgSimpleTestBase{
            .param_1 = MsgSimpleTestBase::Param1_T(),
            .param_2 = MsgSimpleTestBase::Param2_T(),
            .param_3 = MsgSimpleTestBase::Param3_T(),
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_simple_default.cpp.dat";
        write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgSimpleTestBase{
            .param_1 = MsgSimpleTestBase::Param1_T(999),
            .param_2 = MsgSimpleTestBase::Param2_T(true),
            .param_3 = MsgSimpleTestBase::Param3_T(-13),
        };
        msg.print(0);printf("\n");

        auto fn_name = "val_simple_param_set1.cpp.dat";
        write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgSimpleTestFp{
            .param_1 = MsgSimpleTestFp::Param1_T(true),
            .fp = MsgSimpleTestFp::Fp_T(0.1),
        };
        msg.print(0);printf("\n");

        auto fn_name = "val_simple_test_fp.cpp.dat";
        write_or_test(fn_name, msg, arg);
    }
    return 0;
}

