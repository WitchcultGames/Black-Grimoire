extern crate grimoire;

use std::env::args;
use std::path::Path;
use std::fs::File;
//use std::io::{Read, Write};
use std::io::Read;
use std::str::FromStr;
use grimoire::utilities::write_struct;
use grimoire::renderer::Vertex;

fn main() {
    let mut args = args();

    if args.len() != 3 {
        println!("usage: model_builder [ply file name] [Model name]!");
        return;
    }

    let path_string = args.nth(1).unwrap();
    let path = Path::new(path_string.as_str());
    let mut buffer = String::new();

    {
        let mut file;

        match File::open(path) {
            Ok(f) => file = f,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                return;
            }
        }

        match file.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                return;
            }
        }
    }

    let mut header_done = false;
    let mut vertecies_done = false;
    let mut vertex_count = 0;
    let mut verticies: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for l in buffer.lines() {
        let line = match String::from_str(l) {
            Ok(l) => l,
            Err(e) => panic!("{}", e),
        };

        let mut words: Vec<&str> = line.split_whitespace().collect();

        if header_done == false {
            if words[0] == "element" && words[1] == "vertex" {

                vertex_count = match usize::from_str(words[2]) {
                    Ok(n) => n,
                    Err(e) => panic!("Failed to get vertex count: {}!", e),
                };

                verticies.reserve_exact(vertex_count);
            }
            else if words[0] == "end_header" {
                header_done = true;
            }
        } else if vertecies_done == false {
            let position = (f32::from_str(words[0]).unwrap(),
                            f32::from_str(words[1]).unwrap(),
                            f32::from_str(words[2]).unwrap()).into();

            let normal = (f32::from_str(words[3]).unwrap(),
                          f32::from_str(words[4]).unwrap(),
                          f32::from_str(words[5]).unwrap()).into();

            let uv = (f32::from_str(words[6]).unwrap(),
                      f32::from_str(words[7]).unwrap() * -1.0).into();

            verticies.push(Vertex {
                position,
                normal,
                uv,
            });

            if verticies.len() == vertex_count {
                vertecies_done = true;
            }
        } else {
            words.remove(0);

            for index in words.iter() {
                indices.push(u32::from_str(index).unwrap());
            }
        }
    }

    let mut file;
    //let name_string = args.nth(2).unwrap();

    //match File::create(Path::new(name_string.as_str())) {
    match File::create(Path::new("test")) {
        Ok(f) => file = f,
        Err(e) => {
            eprintln!("Failed to create new model file: {}", e);
            return;
        },
    }

    let mut vc = verticies.len() as u32;
    let mut ic = indices.len() as u32;

    match write_struct(&mut file, &mut vc) {
        Ok(_) => (),
        Err(e) => panic!("Failed to write to model file: {}", e),
    }

    match write_struct(&mut file, &mut ic) {
        Ok(_) => (),
        Err(e) => panic!("Failed to write to model file: {}", e),
    }

    for v in verticies.iter_mut() {
        match write_struct(&mut file, &mut *v) {
            Ok(_) => (),
            Err(e) => panic!("Failed to write to model file: {}", e),
        }
    }

    for i in indices.iter_mut() {
        match write_struct(&mut file, &mut *i) {
            Ok(_) => (),
            Err(e) => panic!("Failed to write to model file: {}", e),
        }
    }

    //write!(&mut file, "(\"{}\",\n gl::TRIANGLES,\n &[", "Test").unwrap();

    //for v in verticies.iter() {
    //    write!(&mut file, "Vertex {{\n").unwrap();
    //    write!(&mut file, "      position: Vec3 {{\n").unwrap();
    //    write!(&mut file, "          x: {}_f32,\n", (v.0).0).unwrap();
    //    write!(&mut file, "          y: {}_f32,\n", (v.0).1).unwrap();
    //    write!(&mut file, "          z: {}_f32,\n", (v.0).2).unwrap();
    //    write!(&mut file, "      }},\n").unwrap();
    //    write!(&mut file, "      normal: Vec3 {{\n").unwrap();
    //    write!(&mut file, "          x: {}_f32,\n", (v.1).0).unwrap();
    //    write!(&mut file, "          y: {}_f32,\n", (v.1).1).unwrap();
    //    write!(&mut file, "          z: {}_f32,\n", (v.1).2).unwrap();
    //    write!(&mut file, "      }},\n").unwrap();
    //    write!(&mut file, "      uv: Vec2 {{\n").unwrap();
    //    write!(&mut file, "          x: {}_f32,\n", (v.2).0).unwrap();
    //    write!(&mut file, "          y: -({}_f32),\n", (v.2).1).unwrap();
    //    write!(&mut file, "      }},\n").unwrap();
    //    write!(&mut file, "  }},\n").unwrap();
    //}

    //write!(&mut file, "  ],\n  &[").unwrap();

    //for i in indices.iter() {
    //    write!(&mut file, " {},", i).unwrap();
    //}

    //write!(&mut file, "])").unwrap();
}
