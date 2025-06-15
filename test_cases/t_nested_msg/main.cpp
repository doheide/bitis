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
        auto msg = MsgWithInner();
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_nested_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto inner_msg = MsgEnumOpt{
            .val = MsgEnumOpt::Val_T(1),
            .param_1 = MsgEnumOpt::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
            .param_2 = MsgEnumOpt::Param2_T::create_val(
                MsgEnumOpt::Param2_T::ValT::create_enum<ExampleEnumEnum::E3>())
        };
        auto msg = MsgWithInner{
            .val = MsgWithInner::Val_T(2),
            .imsg = inner_msg,
        };
        printf("\n"); msg.print(0); printf("\n");

        auto fn_name = "val_nested_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto inner = MsgEnumOpt{
            .val = MsgEnumOpt::Val_T(1),
            .param_1 = MsgEnumOpt::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
            .param_2 = MsgEnumOpt::Param2_T::create_val(
                MsgEnumOpt::Param2_T::ValT::create_enum<ExampleEnumEnum::E3>())
        };
        auto msgi = MsgWithInner{
            .val = MsgWithInner::Val_T(2),
            .imsg = inner,
        };
        auto msg = MsgWithTwoInner{
            .val = MsgWithTwoInner::Val_T(47),
            .imsg = msgi,
            .oimsg = MsgWithTwoInner::Oimsg_T::create_none()
        };
        printf("\n"); msg.print(0); printf("\n");
        auto fn_name = "val_nested_two_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgWithInner();
        msg.val = MsgWithInner::Val_T(3);
        msg.imsg.param_1 = MsgEnumOpt::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
        msg.imsg.param_2 = MsgEnumOpt::Param2_T::create_val(
            MsgEnumOpt::Param2_T::ValT::create_enum<ExampleEnumEnum::E7>());
        printf("\n"); msg.print(0); printf("\n");
        auto fn_name = "val_nested_two_val2.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

