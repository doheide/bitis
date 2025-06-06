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

    {
        auto msg = MsgEnumOne();
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_enum_one_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgEnumOne{
            .val = MsgEnumOne::Val_T(3),
            .param_1 = MsgEnumOne::Param1_T::create_enum<SensorSourceEnum::MovementSensor>(),
        };
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_enum_one_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgEnumTwo();
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_enum_two_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgEnumTwo{
            .val = MsgEnumTwo::Val_T(33),
            .param_1 = MsgEnumTwo::Param1_T::create_enum<ExampleEnumEnum::E8>()
        };
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_enum_two_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgEnumOpt();
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_enum_opt_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgEnumOpt{
            .val = MsgEnumOpt::Val_T(3),
            .param_1 = MsgEnumOpt::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
            .param_2 = MsgEnumOpt::Param2_T::create_val(
                MsgEnumOpt::Param2_T::ValT::create_enum<ExampleEnumEnum::E8>())
        };
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_enum_opt_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

