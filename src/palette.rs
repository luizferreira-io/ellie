use ratatui::style::Color;

pub const APP_COLOR1: Color = Color::Rgb(0xDC, 0xDC, 0xDC); // White
pub const APP_COLOR2: Color = Color::Rgb(0xDC, 0xB9, 0x23); // Yellow
pub const APP_COLOR3: Color = Color::Rgb(0x46, 0xC8, 0xD2); // Light cyan
pub const APP_COLOR4: Color = Color::Rgb(0xD2, 0x6E, 0xDC); // Pink
pub const APP_ERROR:  Color = Color::Rgb(0xFF, 0x00, 0x00); // Red 
pub const APP_BKG: Color = Color::Rgb(0x19, 0x32, 0x5A); // Dark blue

pub const APP_HEADER_TAB_ACTIVE: Color = Color::Rgb(0xC8, 0x32, 0x32); // Red
pub const APP_HEADER_TAB_INACTIVE: Color = Color::Rgb(0x12, 0x12, 0x18); // Black
pub const APP_HEADER_BKG: Color = Color::Rgb(0xDC, 0xDC, 0xDC); // White

pub const APP_FOOTER_TEXT: Color = Color::Rgb(0x12, 0x12, 0x18); // Black
pub const APP_FOOTER_SHORTCUT: Color = Color::Rgb(0xC8, 0x32, 0x32); // Red
pub const APP_FOOTER_BKG: Color = Color::Rgb(0xDC, 0xDC, 0xDC); // White

pub const WIDGET_BORDER_ACTIVE: Color = Color::Rgb(0xFF, 0xFF, 0xFF); // Bright white
pub const WIDGET_BORDER_INACTIVE: Color = Color::Rgb(0x00, 0x00, 0x00); // Black
pub const WIDGET_TITLE_ACTIVE: Color = Color::Rgb(0xDC, 0xDC, 0xDC); // White
pub const WIDGET_TITLE_INACTIVE: Color = Color::Rgb(0xA0, 0xA0, 0xA0); // Gray

pub const TABLE_HEADER_TEXT: Color = Color::Rgb(0x46, 0xC8, 0xD2); // Light cyan
pub const TABLE_HEADER_BKG: Color = Color::Rgb(0x28, 0x50, 0x82); // Medium blue
pub const TABLE_SELECTED_COLUMN_BKG: Color = Color::Rgb(0x28, 0x40, 0x72); // Darker blue
pub const TABLE_SELECTED_LINE_TEXT: Color = Color::Rgb(0x12, 0x12, 0x18); // Black
pub const TABLE_SELECTED_LINE_BKG: Color = Color::Rgb(0xDC, 0xDC, 0xDC); // White
