use crate::linalg::{Matrix, Vector};
use crate::material::Material;
use crate::shape::*;

use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

use nom::branch::alt;
use nom::character::complete::{alphanumeric1, anychar, digit1, space1};
use nom::number::complete::double;
use nom::{
    alt, char, complete, do_parse, many_till, map_opt, map_res, named, separated_list0, tag,
};

type VertexNormal = (usize, Option<usize>);

#[derive(Debug, PartialEq)]
enum Obj {
    Vertex {
        x: f64,
        y: f64,
        z: f64,
    },
    Normal {
        x: f64,
        y: f64,
        z: f64,
    },
    Triangles {
        indices: Vec<(VertexNormal, VertexNormal, VertexNormal)>,
    },
    Group {
        name: String,
    },
    Ignored {
        n: u32,
        line: String,
    },
}

#[derive(Debug)]
struct ObjParse {
    objs: Vec<Obj>,
}

#[derive(Debug)]
pub struct ObjParser<'a> {
    path: &'a str,
}

#[rustfmt::skip]
named!(
    parse_usize<&str, usize>,
    map_res!(digit1, |s: &str| s.parse::<usize>())
);

#[rustfmt::skip]
named!(
    parse_vertex<&str, Obj>,
    do_parse!(
           char!('v') >>
           space1     >>
        x: double     >>
           space1     >>
        y: double     >>
           space1     >>
        z: double     >>
        (
            Obj::Vertex { x, y, z}
        )
    )
);

#[rustfmt::skip]
named!(
    parse_normal<&str, Obj>,
    do_parse!(
           tag!("vn") >>
           space1     >>
        x: double     >>
           space1     >>
        y: double     >>
           space1     >>
        z: double     >>
        (
            Obj::Normal { x, y, z}
        )
    )
);

#[rustfmt::skip]
named!(
    parse_face_triplet<&str, (usize, Option<usize>)>,
    do_parse!(
        v: parse_usize                     >>
           char!('/')                      >>
           many_till!(anychar, char!('/')) >>
        n: parse_usize                     >>
        (
            (v, Some(n))
        )
    )
);

named!(
    parse_face<&str, (usize, Option<usize>)>,
    alt!(
        complete!(parse_face_triplet) |
        complete!(map_opt!(parse_usize, |v| Some((v, None))))
    )
);

fn triangulate<T: Copy>(indices: Vec<T>) -> Vec<(T, T, T)> {
    let mut triples = vec![];

    for i in 1..(indices.len() - 1) {
        triples.push((indices[0], indices[i], indices[i + 1]));
    }

    triples
}

#[rustfmt::skip]
named!(
    parse_faces<&str, Obj>,
    do_parse!(
                 char!('f')                           >>
                 space1                               >>
        indices: separated_list0!(space1, parse_face) >>
        (
            Obj::Triangles { indices: triangulate(indices) }
        )
    )
);

#[rustfmt::skip]
named!(
    parse_group<&str, Obj>,
    do_parse!(
              char!('g')    >>
              space1        >>
        name: alphanumeric1 >>
        (
            Obj::Group { name: name.to_string() }
        )
    )
);

impl<'a> ObjParser<'a> {
    pub fn new(path: &'a str) -> ObjParser<'a> {
        ObjParser { path }
    }

    fn parse_line(n: u32, line: &str, obj_parse: &mut ObjParse) {
        match alt((parse_vertex, parse_normal, parse_faces, parse_group))(line) {
            Ok((_, obj)) => {
                obj_parse.objs.push(obj);
            }
            Err(_) => {
                obj_parse.objs.push(Obj::Ignored {
                    n,
                    line: line.to_string(),
                });
            }
        }
    }

    fn parse_lines(&self) -> std::io::Result<ObjParse> {
        let file = fs::File::open(self.path)?;
        let buf_reader = BufReader::new(file);

        let mut obj_parse = ObjParse { objs: vec![] };

        buf_reader.lines().fold(1, |cnt, line| {
            if let Ok(line) = line {
                ObjParser::parse_line(cnt, &line, &mut obj_parse);
            }
            cnt + 1
        });

        Ok(obj_parse)
    }

    pub fn parse_obj(
        &self,
        transform: Matrix,
        material: Material,
    ) -> std::io::Result<(Vec<(u32, String)>, Element)> {
        let mut group = "Default".to_string();
        let mut vertices = vec![];
        let mut normals = vec![];
        let mut ignored = vec![];
        let mut groups: HashMap<String, Vec<Element>> = HashMap::new();
        groups.insert(group.clone(), vec![]);

        let obj_parse = self.parse_lines()?;

        for obj in obj_parse.objs {
            match obj {
                Obj::Vertex { x, y, z } => vertices.push(Vector::point(x, y, z)),
                Obj::Normal { x, y, z } => normals.push(Vector::vector(x, y, z)),
                Obj::Triangles { indices } => {
                    for ((p1, n1), (p2, n2), (p3, n3)) in indices {
                        let triangle = match (n1, n2, n3) {
                            (Some(n1), Some(n2), Some(n3)) => Element::smooth_triangle(
                                ShapeArgs::default(),
                                vertices[p1 - 1],
                                vertices[p2 - 1],
                                vertices[p3 - 1],
                                normals[n1 - 1],
                                normals[n2 - 1],
                                normals[n3 - 1],
                            ),
                            _ => Element::triangle(
                                ShapeArgs::default(),
                                vertices[p1 - 1],
                                vertices[p2 - 1],
                                vertices[p3 - 1],
                            ),
                        };

                        groups.get_mut(&group).unwrap().push(triangle);
                    }
                }
                Obj::Group { name } => {
                    group = name.clone();
                    groups.entry(name).or_insert(vec![]);
                }
                Obj::Ignored { n, line } => {
                    ignored.push((n, line));
                }
            }
        }

        let mut elements = vec![];

        for (_, children) in groups.drain() {
            if children.len() != 0 {
                elements.push(Element::composite(
                    transform,
                    Some(material.clone()),
                    GroupKind::Aggregation,
                    children,
                ));
            }
        }

        Ok((
            ignored,
            if elements.len() == 1 {
                elements.pop().unwrap()
            } else {
                Element::composite(Matrix::id(), None, GroupKind::Aggregation, elements)
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::approx::Approx;

    fn parse_lines(path: &str, contents: &[u8]) -> ObjParse {
        let mut file = fs::File::create(path).unwrap();
        file.write(contents).unwrap();

        let obj_parse = ObjParser::new(path).parse_lines().unwrap();
        let _ = fs::remove_file(path);

        obj_parse
    }

    fn parse_obj(path: &str, contents: &[u8]) -> (Vec<(u32, String)>, Element) {
        let mut file = fs::File::create(path).unwrap();
        file.write(contents).unwrap();

        let result = ObjParser::new(path)
            .parse_obj(Matrix::id(), Material::default())
            .unwrap();
        let _ = fs::remove_file(path);

        result
    }

    #[test]
    fn ignored() {
        let contents = b"There was a young lady named Bright\n\
            who traveled much faster than light.\n\
            She set out one day\n\
            in a relative way,\n\
            and came back the previous night.\n";

        let path = "ignored.obj";
        let obj_parse = parse_lines(path, contents);

        assert!(
            obj_parse.objs.len() == 5
                && obj_parse.objs[0]
                    == Obj::Ignored {
                        n: 1,
                        line: "There was a young lady named Bright".to_string()
                    }
                && obj_parse.objs[1]
                    == Obj::Ignored {
                        n: 2,
                        line: "who traveled much faster than light.".to_string()
                    }
                && obj_parse.objs[2]
                    == Obj::Ignored {
                        n: 3,
                        line: "She set out one day".to_string()
                    }
                && obj_parse.objs[3]
                    == Obj::Ignored {
                        n: 4,
                        line: "in a relative way,".to_string()
                    }
                && obj_parse.objs[4]
                    == Obj::Ignored {
                        n: 5,
                        line: "and came back the previous night.".to_string()
                    }
        )
    }

    #[test]
    fn vertex_records() {
        let contents = b"v -1 1 0\n\
          v -1.0000 0.5000 0.0000\n\
          v 1 0 0\n\
          v 1 1 0\n";

        let path = "vertex_records.obj";
        let obj_parse = parse_lines(path, contents);

        assert!(
            obj_parse.objs.len() == 4
                && obj_parse.objs[0]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[1]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 0.5,
                        z: 0.0
                    }
                && obj_parse.objs[2]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[3]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0
                    }
        )
    }

    #[test]
    fn triangle_faces() {
        let contents = b"v -1 1 0\n\
          v -1 0 0\n\
          v 1 0 0\n\
          v 1 1 0\n\
          \n\
          f 1 2 3\n\
          f 1 3 4\n";

        let path = "triangle_faces.obj";
        let obj_parse = parse_lines(path, contents);

        assert!(
            obj_parse.objs.len() == 7
                && obj_parse.objs[0]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[1]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[2]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[3]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[4]
                    == Obj::Ignored {
                        n: 5,
                        line: "".to_string()
                    }
                && obj_parse.objs[5]
                    == Obj::Triangles {
                        indices: vec![((1, None), (2, None), (3, None))]
                    }
                && obj_parse.objs[6]
                    == Obj::Triangles {
                        indices: vec![((1, None), (3, None), (4, None))]
                    }
        )
    }

    #[test]
    fn triangle_groups() {
        let contents = b"v -1 1 0\n\
            v -1 0 0\n\
            v 1 0 0\n\
            v 1 1 0\n\
            \n\
            g FirstGroup\n\
            f 1 2 3\n\
            g SecondGroup\n\
            f 1 3 4\n";

        let path = "triangle_groups.obj";
        let obj_parse = parse_lines(path, contents);

        assert!(
            obj_parse.objs.len() == 9
                && obj_parse.objs[0]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[1]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[2]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[3]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[4]
                    == Obj::Ignored {
                        n: 5,
                        line: "".to_string()
                    }
                && obj_parse.objs[5]
                    == Obj::Group {
                        name: "FirstGroup".to_string()
                    }
                && obj_parse.objs[6]
                    == Obj::Triangles {
                        indices: vec![((1, None), (2, None), (3, None))]
                    }
                && obj_parse.objs[7]
                    == Obj::Group {
                        name: "SecondGroup".to_string()
                    }
                && obj_parse.objs[8]
                    == Obj::Triangles {
                        indices: vec![((1, None), (3, None), (4, None))]
                    }
        )
    }

    #[test]
    fn vertex_normal_records() {
        let contents = b"vn 0 0 1\n\
            vn 0.707 0 -0.707\n\
            vn 1 2 3\n";

        let path = "vertex_normal_records.obj";
        let obj_parse = parse_lines(path, contents);

        assert!(
            obj_parse.objs.len() == 3
                && obj_parse.objs[0]
                    == Obj::Normal {
                        x: 0.0,
                        y: 0.0,
                        z: 1.0
                    }
                && obj_parse.objs[1]
                    == Obj::Normal {
                        x: 0.707,
                        y: 0.0,
                        z: -0.707
                    }
                && obj_parse.objs[2]
                    == Obj::Normal {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0
                    }
        )
    }

    #[test]
    fn smooth_triangle_faces() {
        let contents = b"v 0 1 0\n\
            v -1 0 0\n\
            v 1 0 0\n\
            \n\
            vn -1 0 0\n\
            vn 1 0 0\n\
            vn 0 1 0\n\
            \n\
            f 1//3 2//1 3//2\n\
            f 1/0/3 2/102/1 3/14/2\n";

        let path = "smooth_triangle_faces.obj";
        let obj_parse = parse_lines(path, contents);

        assert!(
            obj_parse.objs.len() == 10
                && obj_parse.objs[0]
                    == Obj::Vertex {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[1]
                    == Obj::Vertex {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[2]
                    == Obj::Vertex {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[3]
                    == Obj::Ignored {
                        n: 4,
                        line: "".to_string()
                    }
                && obj_parse.objs[4]
                    == Obj::Normal {
                        x: -1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[5]
                    == Obj::Normal {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0
                    }
                && obj_parse.objs[6]
                    == Obj::Normal {
                        x: 0.0,
                        y: 1.0,
                        z: 0.0
                    }
                && obj_parse.objs[7]
                    == Obj::Ignored {
                        n: 8,
                        line: "".to_string()
                    }
                && obj_parse.objs[8]
                    == Obj::Triangles {
                        indices: vec![((1, Some(3)), (2, Some(1)), (3, Some(2)))],
                    }
                && obj_parse.objs[9]
                    == Obj::Triangles {
                        indices: vec![((1, Some(3)), (2, Some(1)), (3, Some(2)))],
                    }
        )
    }

    #[test]
    fn triangulate() {
        let indices = vec![1, 2, 3, 4, 5];
        let triangulation = super::triangulate(indices);

        assert!(
            triangulation.len() == 3
                && triangulation[0] == (1, 2, 3)
                && triangulation[1] == (1, 3, 4)
                && triangulation[2] == (1, 4, 5)
        )
    }

    #[test]
    fn obj() {
        let contents = b"v -1 1 0\n\
            v -1 0 0\n\
            v 1 0 0\n\
            v 1 1 0\n\
            \n\
            g FirstGroup\n\
            f 1 2 3\n\
            g SecondGroup\n\
            f 1 3 4\n";

        let path = "parse_obj.obj";
        let (ignored, element) = parse_obj(path, contents);

        let v1 = Vector::point(-1.0, 1.0, 0.0);
        let v2 = Vector::point(-1.0, 0.0, 0.0);
        let v3 = Vector::point(1.0, 0.0, 0.0);
        let v4 = Vector::point(1.0, 1.0, 0.0);

        let expected1 = Element::composite(
            Matrix::id(),
            None,
            GroupKind::Aggregation,
            vec![
                Element::composite(
                    Matrix::id(),
                    None,
                    GroupKind::Aggregation,
                    vec![Element::triangle(ShapeArgs::default(), v1, v2, v3)],
                ),
                Element::composite(
                    Matrix::id(),
                    None,
                    GroupKind::Aggregation,
                    vec![Element::triangle(ShapeArgs::default(), v1, v3, v4)],
                ),
            ],
        );

        let expected2 = Element::composite(
            Matrix::id(),
            None,
            GroupKind::Aggregation,
            vec![
                Element::composite(
                    Matrix::id(),
                    None,
                    GroupKind::Aggregation,
                    vec![Element::triangle(ShapeArgs::default(), v1, v3, v4)],
                ),
                Element::composite(
                    Matrix::id(),
                    None,
                    GroupKind::Aggregation,
                    vec![Element::triangle(ShapeArgs::default(), v1, v2, v3)],
                ),
            ],
        );

        assert!(
            ignored[0] == (5, "".to_string())
                && (element.approx(&expected1) || element.approx(&expected2))
        )
    }
}
