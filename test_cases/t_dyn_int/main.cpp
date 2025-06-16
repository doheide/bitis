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
        auto msg = MsgWithDynInt();
        msg.print(0); printf("\n");

        auto fn_name = "dyn_int_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgWithDynInt{.val = MsgWithDynInt::Val_T(1), .signed_val = MsgWithDynInt::SignedVal_T(1)};
        msg.print(0); printf("\n");

        auto fn_name = "dyn_int_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgWithDynInt{.val = MsgWithDynInt::Val_T(5), .signed_val = MsgWithDynInt::SignedVal_T(-5)};
        msg.print(0); printf("\n");

        auto fn_name = "dyn_int_val2.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgWithDynInt{.val = MsgWithDynInt::Val_T(8), .signed_val = MsgWithDynInt::SignedVal_T(-8)};
        msg.print(0); printf("\n");

        auto fn_name = "dyn_int_val3.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgWithDynInt{.val = MsgWithDynInt::Val_T(15), .signed_val = MsgWithDynInt::SignedVal_T(-15)};
        msg.print(0); printf("\n");

        auto fn_name = "dyn_int_val4.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }


    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

