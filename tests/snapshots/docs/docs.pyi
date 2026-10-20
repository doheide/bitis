from __future__ import annotations
from enum import Enum

# Version of the bitis compiler that generated this module ("{BITIS_VERSION}").
BITIS_VERSION: str
# Schema hash (0x18fe2976a5287710): first 8 bytes of SHA-256 over the canonical schema.
BITIS_SCHEMA_HASH: int

# CRC-32 helper: CRC-32/ISO-HDLC, same values as zlib.crc32 (not part of the wire format)
def crc32(data: bytes | bytearray | memoryview) -> int: ...
class Crc32:
  def __init__(self) -> None: ...
  def update(self, data: bytes | bytearray | memoryview) -> None: ...
  def finish(self) -> int: ...
  @property
  def value(self) -> int: ...
  def reset(self) -> None: ...



# Enums



# *** Enums for oneof



class OO_Documented_Sel:
  @staticmethod
  def new_flag(flag: bool) -> OO_Documented_Sel: ...
  @staticmethod
  def new_num(num: int) -> OO_Documented_Sel: ...



# *** Messages

class Documented:
  """
  Message doc
  """

  def __init__(self, value: int, kind: Kind, sel: OO_Documented_Sel,) -> None: ...

  @staticmethod
  def default() -> Documented: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Documented:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def value(self) -> int: ...
  @value.setter
  def value(self, v: int) -> None: ...
  @property
  def kind(self) -> Kind: ...
  @kind.setter
  def kind(self, v: Kind) -> None: ...
  @property
  def sel_oo(self) -> OO_Documented_SelEnum: ...
  @property
  def sel_flag(self) -> None | bool: ...
  @sel_flag.setter
  def sel_flag(self, val: bool) -> None: ...
  @property
  def sel_num(self) -> None | int: ...
  @sel_num.setter
  def sel_num(self, val: int) -> None: ...



