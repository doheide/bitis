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

/*    {
        auto msg = MsgSimpleBaseOneInt{
            .param_1 = MsgSimpleBaseOneInt::Param1_T(1122),
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_simple_one_int.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgSimpleBaseThreeInt{
            .param_1 = MsgSimpleBaseThreeInt::Param1_T(1122),
            .param_2 = MsgSimpleBaseThreeInt::Param2_T(3),
            .param_3 = MsgSimpleBaseThreeInt::Param3_T(3),
            .param_4 = MsgSimpleBaseThreeInt::Param4_T(33),
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_simple_three_int.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgSimpleTestBase{
            .param_1 = MsgSimpleTestBase::Param1_T(),
            .param_2 = MsgSimpleTestBase::Param2_T(),
            .param_3 = MsgSimpleTestBase::Param3_T(),
            .name = MsgSimpleTestBase::Name_T()
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_simple_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgSimpleTestBase{
            .param_1 = MsgSimpleTestBase::Param1_T(999),
            .param_2 = MsgSimpleTestBase::Param2_T(true),
            .param_3 = MsgSimpleTestBase::Param3_T(-13),
            .name = MsgSimpleTestBase::Name_T("lalalililolo")
        };
        msg.print(0);printf("\n");

        auto fn_name = "val_simple_param_set1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }*/
    {
        auto msg = MsgSimpleTestFP{
            .param_1 = MsgSimpleTestFP::Param1_T(true),
            .fp = MsgSimpleTestFP::Fp_T(0.1),
            // .fpl = MsgSimpleTestFP::Fpl_T(-1.),
        };
        msg.print(0);printf("\n");

        auto fn_name = "val_simple_test_fp.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    /*
    {
        auto msg = MsgSimpleOpt();
        msg.print(0);printf("\n");

        auto fn_name = "val_simple_opt_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgSimpleOpt{
            .param_1 = MsgSimpleOpt::Param1_T(223),
            .param_2 = MsgSimpleOpt::Param2_T::create_val(BitisBool(true)),
            .param_3 = MsgSimpleOpt::Param3_T::create_val(MsgSimpleOpt::Param3_T::ValT(1234)),
            .param_4 = MsgSimpleOpt::Param4_T::create_none(),
        };
        msg.print(0);printf("\n");

        auto fn_name = "val_simple_opt_valset1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    */

    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

