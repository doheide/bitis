//
// Created by dheide on 12.05.25.
//
#include "messages.h"
#include <iostream>
#include <fstream>
#include <libintl.h>
#include "helper.h"



int main(int argc, char *argv[]){
    int error_counter = 0;

    char *arg = nullptr;
    if (argc > 1) arg = argv[1];

    // MsgKVMapSimple
    {
        auto msg = MsgKVMapSimple();
        msg.print(0); printf("\n");

        auto fn_name = "val_kv_simple_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgKVMapSimple();
        msg.entries.values.push_back(MsgKVSimple{.key = MsgKVSimple::Key_T("lala"),
            .value = MsgKVSimple::Value_T("val1")});
        msg.entries.values.push_back(MsgKVSimple{.key = MsgKVSimple::Key_T("lili"),
            .value = MsgKVSimple::Value_T("valval2")});
        msg.print(0); printf("\n");

        auto fn_name = "val_kv_simple_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    // MsgKVMapOO
    {
        auto msg = MsgKVMapOO();
        msg.print(0); printf("\n");

        auto fn_name = "val_kv_oo_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgKVMapOO();
        msg.entries.values.push_back(MsgKVOO{.key = MsgKVOO::Key_T("lala"),
            .value = MsgKVOO::Value_T().set_oo<MsgKVOO::Value_T::OO_IntVal>(MsgKVOO::Value_T::OO_IntVal::OOType(312))});
        msg.entries.values.push_back(MsgKVOO{.key = MsgKVOO::Key_T("lili"),
            .value = MsgKVOO::Value_T().set_oo<MsgKVOO::Value_T::OO_NumVal>(MsgKVOO::Value_T::OO_NumVal::OOType(0.56789))});
        msg.entries.values.push_back(MsgKVOO{.key = MsgKVOO::Key_T("lolo"),
            .value = MsgKVOO::Value_T().set_oo<MsgKVOO::Value_T::OO_StrVal>(MsgKVOO::Value_T::OO_StrVal::OOType("val1"))});
        msg.print(0); printf("\n");

        auto fn_name = "val_kv_oo_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

