use hexplay::HexViewBuilder;

pub fn test_function() {
    let data: Vec<u8> = (0u8..200u8).collect();

    let view = HexViewBuilder::new(&data[0x00..0x50])
        .address_offset(0x00)
        .row_width(0x10)
        .add_colors(vec![
            (hexplay::color::red(), 0x06..0x0F),
            (hexplay::color::yellow_bold(), 0xF..0x14),
            (hexplay::color::green(), 0x00..0x06),
        ])
        .finish();

    view.print().unwrap();
}
