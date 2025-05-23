//
// Created by dheide on 12.05.25.
//
#include "messages.h"


int main() {
    // ***
    auto data_inner1 = Inner{
        .val = Inner::Val_T(3),
        .opt_bool = Inner::OptBool_T::create_val(BitisBool(true))
    };
    printf("* Inner with optional bool set:\n"); data_inner1.print(0); printf("\n\n");

    // ***
    auto data_inner2 = Inner{
        .val = Inner::Val_T(2),
        .opt_bool = Inner::OptBool_T::create_none()
    };
    printf("* Inner with optional is none:\n"); data_inner2.print(0); printf("\n\n");

    // ***
    auto data_inner_enum = InnerWithEnum{
        .val = Inner::Val_T(2),
        .opt_bool = Inner::OptBool_T::create_none(),
        .num = InnerWithEnum::Num_T::create_enum<NumbersEnum::Three>()
    };
    printf("* Inner enum:\n"); data_inner_enum.print(0); printf("\n\n");

    // ***
    auto p1 = ParamTestWithInner{
        .param_1 = ParamTestWithInner::Param1_T(7),
        .inner = data_inner1
    };
    printf("* ParamWithInner:\n"); p1.print(0); printf("\n\n");

    // ***
    auto p2 = ParamTestWithInner{
        .param_1 = ParamTestWithInner::Param1_T(7),
        .inner = Inner{.val = Inner::Val_T(2), .opt_bool = Inner::OptBool_T::create_none()}
    };
    printf("* ParamWithInner inline init:\n"); p2.print(0); printf("\n\n");

    // ***
    auto p3 = ParamTestWithOo{
        .param_1 = ParamTestWithInner::Param1_T(7),
        .action = ParamTestWithOo::Action_T().set_oo<ParamTestWithOo::Action_T::OO_Val>(
            IntgralWithGivenBitSize<int8_t, 3>(-1) ),
        .num = ParamTestWithOo::Num_T::create_enum<NumbersEnum::Four>()
    };
    printf("* ParamWithOO_1:\n"); p3.print(0); printf("\n\n");

    // ***
    auto p4 = ParamTestWithOo{
        .param_1 = ParamTestWithInner::Param1_T(7),
        .action = ParamTestWithOo::Action_T().set_oo<ParamTestWithOo::Action_T::OO_Inner>(
            Inner{.val = Inner::Val_T(3), .opt_bool = Inner::OptBool_T::create_none()} ),
        .num = ParamTestWithOo::Num_T::create_enum<NumbersEnum::Four>()
    };
    printf("* ParamWithOO_2:\n"); p4.print(0); printf("\n\n");

    // ***
    auto p5 = ParamTestWithOo{
        .param_1 = ParamTestWithInner::Param1_T(7),
        .action = ParamTestWithOo::Action_T().set_oo<ParamTestWithOo::Action_T::OO_Inner>(
            Inner{.val = Inner::Val_T(3), .opt_bool = Inner::OptBool_T::create_val(BitisBool(true))} ),
        .num = ParamTestWithOo::Num_T::create_enum<NumbersEnum::Four>()
    };
    printf("* ParamWithOO_2:\n"); p5.print(0); printf("\n\n");


    return 0;
}

