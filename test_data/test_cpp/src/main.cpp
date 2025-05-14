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
    // auto p3 = ParamTestWithOo{
    //     .param_1 = ParamTestWithInner::Param1_T(7),
    //     .action = ParamTestWithOo::Action_T::create_val(IntgralWithGivenBitSize<int8_t, 3>(-1))
    // };
    // printf("* ParamWithOO:\n"); p3.print(0); printf("\n\n");

    // auto param = ParamTestWithInner{
    //     .param_1 = ParamTestWithInner::Param1_T(5),
    //     .param_2 = ParamTestWithInner::Param2_T::create_val(BitisBool(true)),
    //     .action = ParamTestWithInner::Action_T::create_val(ParamTestWithInner::Action_T::Val_T(-2))
    // };
    // param.print(0);

    return 0;
}

