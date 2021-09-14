use std::collections::HashMap;

pub struct CoffeeCup {
    recipe: CoffeeRecipe,
    volume: f32,
}

impl CoffeeRecipe {
    fn to_string(&self) -> String {
        // let mut recipe: String = !("name: {}", self.name);

        let mut recipe: String;
        for (name, relative_volume) in self.ingredients {
            let str: String = format!("{}", relative_volume);
            // recipe.insert_str(recipe.len()-1, )
            // recipe.insert_str(recipe.len()-1, format!("{} of {}", relative_volume, name))
        }
        // for (key, value) in &*self.ingredients {

        // }
        // map.clear();
    }

    fn getRecipe(&self) -> String {

    }
}

pub struct CoffeeRecipe {
    name: Coffee,
    // simplified example: Late { Water: 0.5, Milk: 0.3, Beans: 0.15, Sugar: 0.05 }
    // todo: make water and beans required components
    ingredients: HashMap<Ingredient, f32>,
}

pub enum Coffee {
    Cappuccino,
    Late,
    Americano,
}

pub enum Ingredient {
    Sugar,
    Milk,
    Water,

    // Coffee beans
    Arabica,
    Robusta,
    Liberica,
    Excelsa,
}
