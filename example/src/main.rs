use utils::prelude::*;

fn main() {
    println!("Hello, world!");

    let graph: Graph<String> = graph! {
        nodes: {
            home = "Home";
            train_station_near_home = "Train station near Home";
            train_station_near_school = "Train station near School";
            school = "School";
            somewhere_else = "Somewhere";
        }
        connections: {
            home <-> train_station_near_home: 3.0;
            train_station_near_home <-> train_station_near_school: 18.0;
            train_station_near_school <-> school: 5.0;
            home -> somewhere_else: 5.0;
        }
    };

    println!("Graph: {:?}", graph);

    for (_, node) in graph.iter() {
        println!("{:?}", node)
    }
}
