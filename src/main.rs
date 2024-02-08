use minefield::Minefield;

mod minefield;

fn main() {
    let mut field = Minefield::generate(20, 20);
    field.print_mines();
    field.foreach_mut(|cell| cell.is_revealed = true);
    field.print_mines();
}
