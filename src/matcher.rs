
use std::collections::HashMap;

use itertools::Itertools;

#[derive(Clone)]
pub struct Path {
    segments: Vec<Segment>,
    trailing: bool,
}

impl Path {
    pub fn new(path: String) -> Path {
        let in_segments: Vec<&str>= path.split("/").collect_vec();
        let trailing = in_segments.last() == Some(&"*");

        let mut num_segments = in_segments.len();
        if trailing {
            num_segments -= 1;
        }

        let mut out_segments = Vec::with_capacity(num_segments);

        for n in 0..num_segments {
            if let Some(segment) = in_segments[n].strip_prefix(":") {
                out_segments.push(Segment::Param(segment.to_owned()))
            } else {
                out_segments.push(Segment::Literal(in_segments[n].to_owned()))
            }
        }
        Path{segments: out_segments, trailing: trailing}
    }

    pub fn match_path(&self, mut s: String) -> Option<Match> {
        let mut params: HashMap<String, String> = HashMap::new();

        for (i_seg, segment) in self.segments.iter().enumerate() {
            let splitted = s.split_once("/");
            let mut current = s.clone();

        
            if let Some((seg, next)) = splitted {
                current = seg.to_string();
                s = next.to_string();
                if i_seg == self.segments.len()-1 && !self.trailing {
                    return None
                }
            } else {
                if i_seg != self.segments.len()-1 || self.trailing {
                    return None
                }
            }

            match segment {
                Segment::Param(param) => { params.insert(param.clone(), current); },
                Segment::Literal(literal) => if literal != &current { return None },
            }

        }

        Some(Match{params: params, trailing: s})
    }

}

#[derive(Debug, PartialEq, Clone)]
enum Segment {
    Param(String),
    Literal(String)
}


pub struct Match {
    pub params: HashMap<String, String>,
    pub trailing: String
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_new() {
        let path = Path::new("users/:id/files/*".to_owned());
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[0], Segment::Literal("users".to_owned()));
        assert_eq!(path.segments[1], Segment::Param("id".to_owned()));
        assert_eq!(path.segments[2], Segment::Literal("files".to_owned()));
        assert!(path.trailing);
    }

    #[test]
    fn test_path_matches() {
        let path = Path::new("users/:id/files/*".to_owned());
        let match_result = path.match_path("users/123/files/foo/bar/baz.txt".to_owned());
        assert!(match_result.is_some());
        let matched = match_result.unwrap();
        assert_eq!(matched.params.get("id"), Some(&"123".to_owned()));
        assert_eq!(matched.trailing, "foo/bar/baz.txt");
    }

}

