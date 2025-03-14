
msg ExampleMessage {
  bool is_active;
  uint_3 value_one;
  optional int_5 signed_value;
  repeated_fixed_3 uint_3 array;
}

enum ErrorFlag(3) {
    DeviceId_Unknown,
    Version_Unknown,
}
msg BSSayHiWithId {
    uint_3 reserved_must_be_two;
    uint_32 device_id;
    uint_5 version;
}
msg BSResponseAfterHi {
    oneof content(2) {
      uint_16 challenge;
      ErrorFlag error_reason;
    }
}

msg BSChallengeResponse {
    repeated_fixed_4 uint_8 proof;
}

msg Error_WrongChallengeResponse { }

msg BSFinalSession {
    oneof content(2) {
      uint_32 session_id;
      Error_WrongChallengeResponse error;
    }
}