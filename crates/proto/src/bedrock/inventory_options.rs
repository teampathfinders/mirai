use util::{BinaryRead, Deserialize, Serialize};

use super::ConnectedPacket;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InventoryLeftTab {
    None,
    Construction,
    Equipment,
    Items,
    Nature,
    Search,
    Survival
}

impl TryFrom<i32> for InventoryLeftTab {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<InventoryLeftTab> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Construction,
            2 => Self::Equipment,
            3 => Self::Items,
            4 => Self::Nature,
            5 => Self::Search,
            6 => Self::Survival,
            v => anyhow::bail!("Invalid left inventory tab: {v}")
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InventoryRightTab {
    None,
    Fullscreen,
    Crafting,
    Armor
}

impl TryFrom<i32> for InventoryRightTab {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<InventoryRightTab> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Fullscreen,
            2 => Self::Crafting,
            3 => Self::Armor,
            v => anyhow::bail!("Invalid right inventory tab: {v}")
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InventoryLayout {
    None,
    Survival,
    RecipeBook,
    Creative
}

impl TryFrom<i32> for InventoryLayout {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<InventoryLayout> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Survival,
            2 => Self::RecipeBook,
            3 => Self::Creative,
            v => anyhow::bail!("Invalid inventory layout: {v}")
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetInventoryOptions {
    pub left_tab: InventoryLeftTab,
    pub right_tab: InventoryRightTab,
    pub recipe_filtering: bool,
    pub inventory_layout: InventoryLayout,
    pub crafting_layout: InventoryLayout
}

impl ConnectedPacket for SetInventoryOptions {
    const ID: u32 = 0x133;
}

impl<'a> Deserialize<'a> for SetInventoryOptions {
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self> {
        let left_tab = InventoryLeftTab::try_from(reader.read_var_i32()?)?;
        let right_tab = InventoryRightTab::try_from(reader.read_var_i32()?)?;
        let recipe_filtering = reader.read_bool()?;
        let inventory_layout = InventoryLayout::try_from(reader.read_var_i32()?)?;
        let crafting_layout = InventoryLayout::try_from(reader.read_var_i32()?)?;

        Ok(Self {
            left_tab, right_tab, recipe_filtering, inventory_layout, crafting_layout
        })
    }
}