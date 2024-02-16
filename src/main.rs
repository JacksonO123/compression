use std::{
    collections::HashMap,
    env,
    fmt::Display,
    fs::{self, File},
    io::{Result, Write},
};

use rand::{thread_rng, Rng};

#[derive(Debug)]
struct Group {
    data: String,
    occurances: u32,
    weight: u32,
}

impl Group {
    fn new(data: String, occurances: u32) -> Self {
        let weight = Self::calculate_weight(&data, occurances);

        Self {
            data,
            weight,
            occurances,
        }
    }

    fn calculate_weight(data: &String, occurances: u32) -> u32 {
        return data.len() as u32 * occurances;
    }
}

struct Mapping {
    free_chars: Vec<char>,
    value_map: HashMap<String, String>,
    ctrl_char: char,
    repeat_char: char,
}

impl Mapping {
    fn new() -> Self {
        let char_str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890~`!@#$%^&*()-=_+[{}]|;:'\"\\,<.>/?";
        let mut free_chars: Vec<char> = char_str.chars().collect();

        let ctrl_char = free_chars.pop().unwrap();
        let repeat_char = free_chars.pop().unwrap();

        Self {
            ctrl_char,
            repeat_char,
            free_chars,
            value_map: HashMap::new(),
        }
    }

    fn next_key(&self) -> String {
        if self.free_chars.len() == 0 {
            panic!("no free chars");
        }

        String::from(self.free_chars.last().unwrap().to_string())
    }

    fn key_used(&mut self) {
        self.free_chars.pop();
    }

    fn insert(&mut self, key: String, value: String) {
        self.value_map.insert(key, value);
    }

    fn stringify(&self, compressed: String) -> String {
        let mut res = if self.value_map.len() > 0 {
            self.ctrl_char.to_string()
        } else {
            String::new()
        };

        for (key, value) in self.value_map.iter() {
            res.push_str(key);
            res.push('=');
            res.push_str(value);
            res.push(self.ctrl_char);
        }

        res.push_str(&compressed);

        res
    }

    fn predict_len(&self, source_len: usize) -> usize {
        let mut key_len = if self.value_map.len() > 0 { 1 } else { 0 };

        for (key, value) in self.value_map.iter() {
            // 2 because 1 for '=' and 1 for ctrl char
            key_len += key.len() + value.len() + 2;
        }

        key_len += source_len;

        key_len
    }
}

impl Display for Mapping {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{:?}", self.value_map);

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    let info = args.get(0).expect("Expected mode");

    match info.as_str() {
        "-g" => {
            let mut rng = thread_rng();

            let output = "data/text.txt";
            let random_words = vec![
                "this",
                "word",
                "random",
                "computer",
                "mouse",
                "food",
                "cheese",
                "book",
                "table",
                "why",
                "would",
                "work",
                "something",
                "idk",
                "epic",
                "more",
                "different",
                "im",
                "done",
            ];
            let num_words = 200;
            let mut random_str = String::new();

            for i in 0..num_words {
                let index = rng.gen_range(0..random_words.len());
                random_str.push_str(&random_words[index].clone());

                if i < num_words - 1 {
                    random_str.push(' ');
                }
            }

            let mut file = File::create(output)?;
            file.write_all(random_str.as_bytes())?;
        }
        _ => {
            let outfile_name = info.to_owned() + ".smol";

            let mut source = fs::read_to_string(info)?;
            let input = source.clone();

            let mut mapping = Mapping::new();

            compress(&mut mapping, &mut source);

            println!("{}", mapping);
            println!("input: {}", input);

            let res = mapping.stringify(source);
            println!("{}", res);

            let mut file = File::create(outfile_name)?;
            file.write_all(res.as_bytes())?;
        }
    };

    Ok(())
}

fn compress(mapping: &mut Mapping, source: &mut String) {
    collapse_repeats(mapping, source);

    loop {
        let used = create_groups(mapping, source);
        println!("{}", mapping.predict_len(source.len()));

        if !used {
            break;
        }
    }
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

fn create_groups(mapping: &mut Mapping, source: &mut String) -> bool {
    let mut best_group: Option<Group> = None;

    let mut start = 0;
    let mut end = source.len() / 4;

    while start != end - 1 {
        let slice = String::from(&source[start..end]);

        let count = find_occurances(source, &slice);

        if count > 1 {
            let new_group = Group::new(slice.clone().to_string(), count);
            if let Some(ref mut best_group) = &mut best_group {
                if new_group.weight > best_group.weight {
                    *best_group = new_group;
                }
            } else {
                best_group = Some(new_group);
            }
        }

        if end == source.len() {
            end = source.len() - start - 1;
            start = 0;
        } else {
            start += 1;
            end += 1;
        }
    }

    if let Some(group) = best_group {
        collapse_groups(mapping, source, &group.data);
        let next_key = mapping.next_key();
        let added_len =
            3 + group.data.len() as u32 + (next_key.len() as u32 * (group.occurances + 1));
        let removed_len = group.data.len() as u32 * group.occurances;

        if added_len < removed_len {
            mapping.key_used();
            mapping.insert(next_key.clone(), group.data.clone());
            replace_all(source, &group.data, &next_key);

            return true;
        }
    }

    false
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

fn find_occurances(source: &str, substr: &str) -> u32 {
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
