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

    auto inner1 = Inner{ .val2 = Inner::Val2_T(3)};
    auto inner2 = Inner{ .val2 = Inner::Val2_T(1)};

    // *** MsgFixedBaseArray
    {
        auto msg = MsgFixedBaseArray();
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_array_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgFixedBaseArray{
            .param_1 = MsgFixedBaseArray::Param1_T::create_enum<SensorSourceEnum::TemperaturSensor>(),
            .val1 = MsgFixedBaseArray::Val1_T({
                MsgFixedBaseArray::Val1_T::ValT(1), MsgFixedBaseArray::Val1_T::ValT(2), MsgFixedBaseArray::Val1_T::ValT(3)}),
            .val2 = MsgFixedBaseArray::Val2_T({MsgFixedBaseArray::Val2_T::ValT(-2),
                MsgFixedBaseArray::Val2_T::ValT(2), MsgFixedBaseArray::Val2_T::ValT(0)}),
            .val3 = MsgFixedBaseArray::Val3_T({MsgFixedBaseArray::Val3_T::ValT(true),
                MsgFixedBaseArray::Val3_T::ValT(false), MsgFixedBaseArray::Val3_T::ValT(true)}),
            .val4 = MsgFixedBaseArray::Val4_T({MsgFixedBaseArray::Val4_T::ValT(-1),
                MsgFixedBaseArray::Val4_T::ValT(123), MsgFixedBaseArray::Val4_T::ValT(10)}),
            .val5 = MsgFixedBaseArray::Val5_T({MsgFixedBaseArray::Val5_T::ValT(1.1),
                MsgFixedBaseArray::Val5_T::ValT(2.2), MsgFixedBaseArray::Val5_T::ValT(123.456)}),
            .val6 = MsgFixedBaseArray::Val6_T({MsgFixedBaseArray::Val6_T::ValT(1.1),
                MsgFixedBaseArray::Val6_T::ValT(-1.1), MsgFixedBaseArray::Val6_T::ValT(1.2)}),
            .val7 = MsgFixedBaseArray::Val7_T({MsgFixedBaseArray::Val7_T::ValT::create_enum<SensorSourceEnum::MovementSensor>(),
                MsgFixedBaseArray::Val7_T::ValT::create_enum<SensorSourceEnum::MovementSensor>(),
                MsgFixedBaseArray::Val7_T::ValT::create_enum<SensorSourceEnum::TemperaturSensor>()}),
            .val8 = MsgFixedBaseArray::Val8_T({inner1, inner2, inner1})
        };
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_array_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    // *** MsgDynBaseArray
    {
        auto msg = MsgDynBaseArray();
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_dynarray_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgDynBaseArray{
            .ee = MsgDynBaseArray::Ee_T::create_enum<ExampleEnumEnum::E5>(),
            .val1 = MsgDynBaseArray::Val1_T({
                MsgDynBaseArray::Val1_T::ValT(1), MsgDynBaseArray::Val1_T::ValT(2), MsgDynBaseArray::Val1_T::ValT(3)}),
            .val2 = MsgDynBaseArray::Val2_T({MsgDynBaseArray::Val2_T::ValT(-2),
                MsgDynBaseArray::Val2_T::ValT(2), MsgDynBaseArray::Val2_T::ValT(0)}),
            .val3 = MsgDynBaseArray::Val3_T({MsgDynBaseArray::Val3_T::ValT(true),
            MsgDynBaseArray::Val3_T::ValT(false), MsgDynBaseArray::Val3_T::ValT(true)}),
            .val4 = MsgDynBaseArray::Val4_T({MsgDynBaseArray::Val4_T::ValT(-1),
                MsgDynBaseArray::Val4_T::ValT(123), MsgDynBaseArray::Val4_T::ValT(10)}),
            .val5 = MsgDynBaseArray::Val5_T({MsgDynBaseArray::Val5_T::ValT(1.1),
                MsgDynBaseArray::Val5_T::ValT(2.2), MsgDynBaseArray::Val5_T::ValT(123.456)}),
            .val6 = MsgDynBaseArray::Val6_T({MsgDynBaseArray::Val6_T::ValT(1.1),
                MsgDynBaseArray::Val6_T::ValT(-1.1), MsgDynBaseArray::Val6_T::ValT(1.2)}),
            .val7 = MsgDynBaseArray::Val7_T({MsgDynBaseArray::Val7_T::ValT::create_enum<SensorSourceEnum::MovementSensor>(),
                MsgDynBaseArray::Val7_T::ValT::create_enum<SensorSourceEnum::MovementSensor>(),
                MsgDynBaseArray::Val7_T::ValT::create_enum<SensorSourceEnum::TemperaturSensor>()}),
            .val8 = MsgDynBaseArray::Val8_T({inner1, inner1, inner2})
        };
        msg.val1.values.emplace_back(4);
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_dynarray_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgDynBaseArray();
        for (int i=0; i!=13; i++) { msg.val1.values.emplace_back(i&7); }
        for (int i=0; i!=23; i++) { msg.val2.values.emplace_back(i&7); }
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_dynarray_val2.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    // *** MsgLaFixedBaseArray
    {
        auto msg = MsgLargeFixedArray();
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_large_array_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    {
        auto msg = MsgLargeFixedArray();
        for (int i=0; i!=msg.val1.values.size(); i++) {
            msg.val1.values[i] = MsgFixedBaseArray::Val1_T::ValT((i+1) & 7);
        }
        for (int i=0; i!=msg.val2.values.size(); i++) {
            char ii = i & 7;
            msg.val2.values[i] = MsgFixedBaseArray::Val2_T::ValT((1-(ii&1)*2)*(ii>>1));
        }
        for (int i=0; i!=msg.val2.values.size(); i++) {
            msg.val3.values[i] = MsgFixedBaseArray::Val3_T::ValT((i&2)==2);
        }
        printf("\n"); msg.print(0); printf("\n");

        const auto fn_name = "val_large_array_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }


    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

