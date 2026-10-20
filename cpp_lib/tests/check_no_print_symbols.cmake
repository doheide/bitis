# Fails if the objects compiled with BITIS_NO_PRINT reference printf / puts / putchar.
#   cmake -DNM=<nm> -DOBJECTS=<obj;obj...> -P check_no_print_symbols.cmake
foreach(obj IN LISTS OBJECTS)
    execute_process(COMMAND ${NM} -u ${obj} OUTPUT_VARIABLE syms RESULT_VARIABLE rc)
    if (NOT rc EQUAL 0)
        message(FATAL_ERROR "${NM} failed on ${obj}")
    endif()
    if (syms MATCHES "printf|puts|putchar")
        message(FATAL_ERROR "${obj} references print functions:\n${syms}")
    endif()
endforeach()
