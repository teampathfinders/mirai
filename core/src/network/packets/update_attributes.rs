use std::ops::Range;
use util::bytes::{BinaryWrite, MutableBuffer};
use util::Serialize;
use crate::network::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum AttributeOperation {
    Addition,
    MultiplyBase,
    MultiplyTotal,
    Cap
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(i32)]
pub enum AttributeOperand {
    Max,
    Min,
    Current
}

#[derive(Debug)]
pub struct AttributeModifier<'a> {
    id: &'a str,
    name: &'a str,
    amount: f32,
    operation: AttributeOperation,
    operand: AttributeOperand,
    serializable: bool
}

impl<'a> Serialize for AttributeModifier<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_str(self.id)?;
        buffer.write_str(self.name)?;
        buffer.write_f32_le(self.amount)?;
        buffer.write_i32_le(self.operation as i32)?;
        buffer.write_i32_le(self.operand as i32)?;
        buffer.write_bool(self.serializable)
    }
}

#[derive(Debug)]
pub struct Attribute<'a> {
    pub name: &'a str,
    pub value: f32,
    pub range: Range<f32>,
    pub default: f32,
    pub modifiers: Vec<AttributeModifier<'a>>
}

impl<'a> Serialize for Attribute<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_f32_le(self.range.start)?;
        buffer.write_f32_le(self.range.end)?;
        buffer.write_f32_le(self.value)?;
        buffer.write_f32_le(self.default)?;
        buffer.write_str(self.name)?;

        buffer.write_var_u32(self.modifiers.len() as u32)?;
        for modifier in &self.modifiers {
            modifier.serialize(buffer)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateAttributes<'a> {
    pub runtime_id: u64,
    pub attributes: Vec<Attribute<'a>>,
    pub tick: u64
}

impl<'a> ConnectedPacket for UpdateAttributes<'a> {
    const ID: u32 = 0x1d;
}

impl<'a> Serialize for UpdateAttributes<'a> {
    fn serialize(&self, buffer: &mut MutableBuffer) -> anyhow::Result<()> {
        buffer.write_var_u64(self.runtime_id)?;

        buffer.write_var_u32(self.attributes.len() as u32)?;
        for attribute in &self.attributes {
            attribute.serialize(buffer)?;
        }

        buffer.write_var_u64(self.tick)
    }
}