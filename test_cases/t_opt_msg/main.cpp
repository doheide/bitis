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

    // *** MsgFixedBaseArray
    {
        auto msg = MsgOpt();
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_opt_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgOpt{
            .param_1 = MsgOpt::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
            .val1 = MsgOpt::Val1_T::create_val(MsgOpt::Val1_T::ValT(3)),
            .val2 = MsgOpt::Val2_T::create_val(MsgOpt::Val2_T::ValT(-2)),
            .val3 = MsgOpt::Val3_T::create_val(MsgOpt::Val3_T::ValT(true)),
            .val4 = MsgOpt::Val4_T::create_val(MsgOpt::Val4_T::ValT(-1)),
            .val5 = MsgOpt::Val5_T::create_val(MsgOpt::Val5_T::ValT(1.1)),
            .val6 = MsgOpt::Val6_T::create_val(MsgOpt::Val6_T::ValT(1.1)),
            .val7 = MsgOpt::Val7_T::create_val(MsgOpt::Val7_T::ValT::create_enum<SensorSourceEnum::MovementSensor>()),
        };
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_opt_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

