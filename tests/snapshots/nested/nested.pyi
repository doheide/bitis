from __future__ import annotations
from enum import Enum

# Version of the bitis compiler that generated this module ("{BITIS_VERSION}").
BITIS_VERSION: str
# Schema hash (0xf9d3f02b9ff943e5): first 8 bytes of SHA-256 over the canonical schema.
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



class OO_Top_Outer:
  @staticmethod
  def new_leaf(leaf: Leaf) -> OO_Top_Outer: ...
  @staticmethod
  def new_inner(inner: OO_Outer_Inner) -> OO_Top_Outer: ...


class OO_Outer_Inner:
  @staticmethod
  def new_x(x: bool) -> OO_Outer_Inner: ...
  @staticmethod
  def new_e(e: E) -> OO_Outer_Inner: ...



# *** Messages

class Top:
  def __init__(self, m: Middle, outer: OO_Top_Outer,) -> None: ...

  @staticmethod
  def default() -> Top: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Top:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def m(self) -> Middle: ...
  @m.setter
  def m(self, v: Middle) -> None: ...
  @property
  def outer_oo(self) -> OO_Top_OuterEnum: ...
  @property
  def outer_leaf(self) -> None | Leaf: ...
  @outer_leaf.setter
  def outer_leaf(self, val: Leaf) -> None: ...
  @property
  def outer_inner(self) -> None | OO_Outer_Inner: ...
  @outer_inner.setter
  def outer_inner(self, val: OO_Outer_Inner) -> None: ...



class Middle:
  def __init__(self, leaves: list[Leaf], e: E,) -> None: ...

  @staticmethod
  def default() -> Middle: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Middle:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def leaves(self) -> list[Leaf]: ...
  @leaves.setter
  def leaves(self, v: list[Leaf]) -> None: ...
  @property
  def e(self) -> E: ...
  @e.setter
  def e(self, v: E) -> None: ...



class Leaf:
  def __init__(self, v: int,) -> None: ...

  @staticmethod
  def default() -> Leaf: ...

  def serialize(self) -> bytes:
    """
    Serialized the class instance to binary data

    :return: bytes representing the serialized class instance
    """
  @staticmethod
  def deserialize(data: bytes) -> Leaf:
    """
    Deserializes bytes to a python object

    :param data: binary data
    :return: An instance of the class with attributes set from the byte data
    """

  @property
  def v(self) -> int: ...
  @v.setter
  def v(self, v: int) -> None: ...



