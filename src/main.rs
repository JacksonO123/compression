use std::collections::HashMap;

struct Mapping {
    free_chars: Vec<char>,
    map_frames: Vec<HashMap<String, String>>,
    ctrl_char: char,
    repeat_char: char,
}

impl Mapping {
    fn new(source: &String) -> Self {
        // let mut free_chars = find_free_chars(source);
        // let mut free_chars = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g'];
        let mut free_chars = vec![
            '!', '@', '#', '$', '%', '^', '|', '*', '(', ')', '-', '+', '/', ',', '.', '<', '>',
            '[', ']', '{', '}', '?', '&',
        ];

        let ctrl_char = free_chars.pop().unwrap();
        let repeat_char = free_chars.pop().unwrap();

        Self {
            ctrl_char,
            repeat_char,
            free_chars,
            map_frames: vec![],
        }
    }

    fn next_key(&mut self) -> String {
        if self.free_chars.len() == 0 {
            panic!("no free chars");
        }

        return String::from(self.free_chars.pop().unwrap());
    }

    fn insert(&mut self, key: &String, value: &String) {
        if let Some(frame) = self.map_frames.get_mut(0) {
            frame.insert(key.clone(), value.clone());
        } else {
            let mut frame: HashMap<String, String> = HashMap::new();
            frame.insert(key.clone(), value.clone());
            self.map_frames.push(frame);
        }
    }

    fn print_frame(&self) {
        println!("{:?}", self.map_frames);
    }

    fn stringify(&self, compressed: String) -> String {
        let mut res = if self.map_frames.len() > 0 {
            self.ctrl_char.to_string()
        } else {
            String::new()
        };

        for frame in self.map_frames.iter() {
            for (key, value) in frame.iter() {
                res.push_str(key);
                res.push('=');
                res.push_str(value);
                res.push(self.ctrl_char);
            }
        }

        res.push_str(&compressed);

        res
    }
}

fn main() {
    // let mut source = String::from("aaaaaaabababababababababababab");
    let mut source = String::from("abababab");
    // let mut source = String::from(
    //     "there are many jobs so many jobs idk what else to so yeah jobs more times say jobs",
    // );
    let input = source.clone();

    let mut mapping = Mapping::new(&source);

    compress(&mut mapping, &mut source);
    compress(&mut mapping, &mut source);

    println!("input: {}", input);
    mapping.print_frame();

    let res = mapping.stringify(source);
    println!("> {}", res);
}

fn compress(mapping: &mut Mapping, source: &mut String) {
    collapse_repeats(mapping, source);
    println!("{}", source);
    create_groups(mapping, source);
    println!("{}", source);
    mapping.print_frame();
}

fn collapse_repeats(mapping: &mut Mapping, source: &mut String) {
    let mut source_chars: Vec<char> = source.chars().collect();

    let mut i = 0;
    while i < source.len() {
        let mut j = i + 1;

        while j < source.len() && source_chars[j] == source_chars[i] {
            j += 1;
        }

        let diff = j - i;

        if diff <= 2 {
            i += 1;
            continue;
        }

        let substr = &source[i..j];
        let mut res_str = mapping.repeat_char.to_string();
        res_str.push_str(&diff.to_string());
        res_str.push(source_chars[i]);
        *source = source.replace(substr, &res_str);

        source_chars = source.chars().collect();

        i += res_str.len() + 1;
    }
}

fn create_groups(mapping: &mut Mapping, source: &mut String) {
    let mut start = 0;
    let mut end = 2;

    while end - start < source.len() / 2 {
        let slice = String::from(&source[start..end]);

        let count = find_occurances(source, &slice);

        if count > 1 {
            collapse_groups(mapping, source, &slice);
            // 2 because after that is starts mattering ig idk
            if slice.len() > 2 {
                let next_key = mapping.next_key();
                mapping.insert(&next_key, &slice.to_string());
                replace_all(source, &slice.to_string(), &next_key);
            }

            if end > source.len() {
                let diff = end - source.len() - 1;

                if diff > start {
                    start = 0;
                } else {
                    start -= diff;
                }

                end = source.len();

                continue;
            }
        }

        if end == source.len() {
            end = source.len() - start + 1;
            start = 0;
        } else {
            start += 1;
            end += 1;
        }
    }
}

fn collapse_groups(mapping: &mut Mapping, source: &mut String, target: &String) {
    let start_index = source.find(target).unwrap();
    let mut index = start_index;
    let mut slice = &source[index + target.len()..];
    index += target.len();

    let mut count = 1;

    while let Some(new_index) = slice.find(target) {
        if new_index != 0 {
            break;
        }
        count += 1;
        index += target.len();
        slice = &source[index..];
    }

    if count > 1 {
        let mut replace_val = mapping.ctrl_char.to_string();
        replace_val.push_str(&count.to_string());
        replace_val.push(mapping.ctrl_char);
        replace_val.push_str(target);
        *source = source.replace(&source[start_index..index], &replace_val)
    }
}

fn replace_all(source: &mut String, target: &String, new_value: &String) {
    while source.contains(target) {
        *source = source.replace(target, new_value);
    }
}

fn find_occurances(source: &str, substr: &str) -> usize {
    let mut count = 0;

    let source_chars: Vec<char> = source.chars().collect();
    let substr_chars: Vec<char> = substr.chars().collect();

    let mut i = 0;
    'outer: while i < source_chars.len() {
        for j in 0..substr_chars.len() {
            if i + j >= source_chars.len() || source_chars[i + j] != substr_chars[j] {
                i += 1;
                continue 'outer;
            }
        }

        i += substr.len();
        count += 1;
    }

    count
}

fn find_free_chars(source: &str) -> Vec<char> {
    let mut res = vec![];

    for i in (0..128 as u8).rev() {
        if (i as char).is_ascii() && !source.contains(i as char) {
            println!("{i}");
            res.push(i as char);
        }
    }

    res
}
