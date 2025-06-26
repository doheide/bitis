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

    // MsgOoSimpleBase
    {
        auto msg = MsgOOSimpleBase();
        msg.print(0); printf("\n");

        auto fn_name = "val_oosimple_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgOOSimpleBase{
            .id = MsgOOSimpleBase::Id_T(53),
            .value = MsgOOSimpleBase::Value_T().set_oo<MsgOOSimpleBase::Value_T::OO_Number>(
                MsgOOSimpleBase::Value_T::OO_Number::OOType(1.23))
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_oosimple_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgOOSimpleBase{
            .id = MsgOOSimpleBase::Id_T(54),
            .value = MsgOOSimpleBase::Value_T().set_oo<MsgOOSimpleBase::Value_T::OO_Int>(
                OO_MsgOoSimpleBase_Value::OO_Int::OOType(3))
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_oosimple_val2.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgOOSimpleBase{
            .id = MsgOOSimpleBase::Id_T(55),
            .value = MsgOOSimpleBase::Value_T().set_oo<MsgOOSimpleBase::Value_T::OO_TrueFalse>(
                OO_MsgOoSimpleBase_Value::OO_TrueFalse::OOType(true))
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_oosimple_val3.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    // MsgOONestedBase
    {
        auto msg = MsgOONestedBase();
        msg.print(0); printf("\n");

        auto fn_name = "val_oonested_default.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgOONestedBase{
            .id = MsgOONestedBase::Id_T(2),
            .value = MsgOONestedBase::Value_T().set_oo<MsgOONestedBase::Value_T::OO_Inner>(
                MsgSimpleBaseOneInt{.param_1 = MsgSimpleBaseOneInt::Param1_T(1111)})
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_oonested_val1.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }
    {
        auto msg = MsgOONestedBase{
            .id = MsgOONestedBase::Id_T(3),
            .value = MsgOONestedBase::Value_T().set_oo<MsgOONestedBase::Value_T::OO_Number>(
                MsgOONestedBase::Value_T::OO_Number::OOType(123.456))
        };
        msg.print(0); printf("\n");

        auto fn_name = "val_oonested_val2.cpp.dat";
        error_counter += write_or_test(fn_name, msg, arg);
    }

    // ********************************************************************************
    // The following messages are not allowed as these can (currently) not be implemented
    // in python --> Combining oneofs with modifiers is not allowed to ensure python compability
    //
    // // MsgOONestedArray
    // {
    //     auto msg = MsgOONestedArray();
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_oonestedarray_default.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    // {
    //     auto msg = MsgOONestedArray();
    //     msg.values.values.emplace_back(MsgOONestedBase{
    //         .id = MsgOONestedBase::Id_T(2),
    //         .value = MsgOONestedBase::Value_T().set_oo<MsgOONestedBase::Value_T::OO_Inner>(
    //             MsgSimpleBaseOneInt{.param_1 = MsgSimpleBaseOneInt::Param1_T(1111)})
    //     });
    //     msg.values.values.emplace_back();
    //     msg.values.values.emplace_back();
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_oonestedarray_val1.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    //
    // // MsgRepeatedFixedOOBase
    // {
    //     auto msg = MsgRepeatedFixedOOBase();
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_repeatedoo_default.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    // {
    //     auto msg = MsgRepeatedFixedOOBase{
    //         .id = MsgRepeatedFixedOOBase::Id_T(1),
    //         .value = MsgRepeatedFixedOOBase::Value_T({
    //             MsgRepeatedFixedOOBase::Value_T::ValT(), MsgRepeatedFixedOOBase::Value_T::ValT(),
    //             MsgRepeatedFixedOOBase::Value_T::ValT().set_oo<MsgRepeatedFixedOOBase::Value_T::ValT::OO_TrueFalse>(
    //             true)
    //         })
    //     };
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_repeatedoo_val1.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    //
    // // MsgRepeatedFixedOOBase
    // {
    //     auto msg = MsgRepeatedDynOOBase();
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_repeateddynoo_default.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    // {
    //     auto msg = MsgRepeatedDynOOBase();
    //     msg.id = MsgRepeatedDynOOBase::Id_T(65);
    //     msg.value.values.emplace_back(MsgRepeatedDynOOBase::Value_T::ValT()
    //         .set_oo<OO_MsgRepeatedDynOoBase_Value::OO_TrueFalse>(true));
    //     msg.value.values.emplace_back(MsgRepeatedDynOOBase::Value_T::ValT()
    //         .set_oo<OO_MsgRepeatedDynOoBase_Value::OO_Int>(
    //             OO_MsgRepeatedDynOoBase_Value::OO_Int::OOType(34)));
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_repeateddynoo_val1.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    //
    // // MsgRepeatedFixedOOBase
    // {
    //     auto msg = MsgOptionalOOBase();
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_optionaloo_default.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }
    // {
    //     auto msg = MsgOptionalOOBase{
    //         .id = MsgOptionalOOBase::Id_T(83),
    //         .value = MsgOptionalOOBase::Value_T::create_val(MsgOptionalOOBase::Value_T::ValT()
    //             .set_oo<MsgOptionalOOBase::Value_T::ValT::OO_TrueFalse>(true))
    //     };
    //     msg.print(0); printf("\n");
    //
    //     auto fn_name = "val_optionaloo_val1.cpp.dat";
    //     error_counter += write_or_test(fn_name, msg, arg);
    // }


    printf("\nTotal_errors: %d\n", error_counter);
    return error_counter;
}

