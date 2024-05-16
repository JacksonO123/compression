use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::{Result, Write},
};

use rand::{thread_rng, Rng};

#[derive(Debug)]
struct Pair {
    key: String,
    value: String,
}

#[derive(Debug)]
enum DecompressState {
    Mapping,
    Repeat,
}

#[derive(Debug)]
struct Group {
    data: String,
    occurances: usize,
    weight: usize,
}

impl Group {
    fn new(data: String, occurances: usize) -> Self {
        let weight = Self::calculate_weight(&data, occurances);

        Self {
            data,
            weight,
            occurances,
        }
    }

    fn calculate_weight(data: &String, occurances: usize) -> usize {
        data.len() * occurances
    }
}

struct Mapping {
    free_chars: Vec<char>,
    values: Vec<Pair>,
    ctrl_char: char,
    repeat_char: char,
}

impl Mapping {
    fn new(source: &str) -> Self {
        let mut free_chars = find_free_chars(source);

        let ctrl_char = free_chars.pop().unwrap();
        let repeat_char = free_chars.pop().unwrap();

        Self {
            ctrl_char,
            repeat_char,
            free_chars,
            values: vec![],
        }
    }

    fn set_ctrl_char(&mut self, ctrl_char: char) {
        self.ctrl_char = ctrl_char;
    }

    fn set_repeat_char(&mut self, repeat_char: char) {
        self.repeat_char = repeat_char;
    }

    fn next_key(&self) -> String {
        if self.free_chars.is_empty() {
            panic!("no free chars");
        }

        self.free_chars.last().unwrap().to_string()
    }

    fn key_used(&mut self) {
        self.free_chars.pop();
    }

    fn insert(&mut self, key: String, value: String) {
        self.values.push(Pair { key, value });
    }

    fn stringify(&self, compressed: String) -> String {
        let mut res = if !self.values.is_empty() {
            let mut temp = String::from(self.repeat_char);
            temp.push(self.ctrl_char);
            temp.push(self.ctrl_char);
            temp
        } else {
            let mut temp = String::from(self.repeat_char);
            temp.push(self.ctrl_char);
            temp
        };

        for (i, pair) in self.values.iter().enumerate() {
            if i != 0 {
                res.push(self.ctrl_char);
            }

            res.push_str(&pair.key);
            res.push('=');
            res.push_str(&pair.value);
            res.push(self.ctrl_char);
        }

        res.push_str(&compressed);

        res
    }

    fn predict_len(&self, source_len: usize) -> usize {
        let mut key_len = if !self.values.is_empty() { 3 } else { 2 };

        for pair in self.values.iter() {
            // 3 because 1 for '=' and 2 for ctrl char
            key_len += pair.key.len() + pair.value.len() + 3;
        }

        key_len += source_len;

        key_len
    }
}

impl Display for Mapping {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(_f, "cmd: {}, repeat: {}", self.ctrl_char, self.repeat_char)?;
        writeln!(_f, "{:#?}", self.values)?;

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
            let num_words = 1000;
            let mut random_str = String::new();

            for i in 0..num_words {
                let index = rng.gen_range(0..random_words.len());
                random_str.push_str(random_words[index].clone());

                if i < num_words - 1 {
                    random_str.push(' ');
                }
            }

            let mut file = File::create(output)?;
            file.write_all(random_str.as_bytes())?;
        }
        "-d" => {
            let input_file = args.get(1).expect("Expected input file for decompression");
            let output_file = args.get(2);

            let data = fs::read_to_string(input_file)?;
            println!("decompressing: {}", data);
            let res = decompress(&data);

            if let Some(output_file) = output_file {
                let mut file = File::create(output_file)?;
                file.write_all(res.as_bytes())?;
            } else {
                println!("{}", res);
            }
        }
        _ => {
            let outfile_name = info.to_owned() + ".smol";

            let source = fs::read_to_string(info)?;

            let mut mapping = Mapping::new(&source);

            let compressed_source = compress(&mut mapping, &source);

            println!("{}", source);

            let res = mapping.stringify(compressed_source);
            println!("{}", res);

            let mut file = File::create(outfile_name)?;
            file.write_all(res.as_bytes())?;
        }
    };

    Ok(())
}

fn decompress(source: &str) -> String {
    let mut source_clone = source.to_owned();

    let mapping = expand_gen_map(&mut source_clone);
    expand_repeat_chars(&mapping, &mut source_clone);
    replace_mapping(&mapping, &mut source_clone);

    source_clone
}

fn expand_repeat_chars(mapping: &Mapping, source: &mut String) {
    let mut res = String::new();
    let mut buf = String::new();
    let chars: Vec<char> = source.chars().collect();

    let mut i = 0;
    while i < source.len() {
        if chars[i] == mapping.repeat_char {
            i += 1;

            res.push_str(&buf);
            buf.clear();

            while chars[i] != mapping.repeat_char {
                buf.push(chars[i]);
                i += 1;
            }

            i += 2;

            let to_repeat = buf.parse().expect("Error getting repeat char number");
            res.push_str(&chars[i - 1].to_string().repeat(to_repeat));
            buf.clear();
        } else {
            res.push(chars[i]);
        }

        i += 1;
    }

    *source = res;
}

fn replace_mapping(mapping: &Mapping, source: &mut String) {
    for pair in mapping.values.iter().rev() {
        while source.contains(&pair.key) {
            *source = source.replace(&pair.key, &pair.value);
        }
    }
}

fn expand_gen_map(source: &mut String) -> Mapping {
    let mut res = String::new();
    let mut mapping = Mapping::new(source);

    let chars: Vec<char> = source.chars().collect();

    let repeat_char = chars[0];
    let ctrl_char = chars[1];

    mapping.set_ctrl_char(ctrl_char);
    mapping.set_repeat_char(repeat_char);

    println!("repeat: {}, ctrl: {}", repeat_char, ctrl_char);

    let mut buf = String::new();
    let mut current_key = String::new();
    let mut state: Option<DecompressState> = None;
    let mut to_repeat = 1;

    let mut i = 2;
    while i < chars.len() {
        if chars[i] == ctrl_char {
            i += 1;

            res.push_str(&buf);
            buf.clear();

            loop {
                if chars[i] == '=' {
                    current_key = buf.clone();
                    buf.clear();
                    state = Some(DecompressState::Mapping);
                } else if chars[i] == mapping.ctrl_char {
                    if let Some(state) = &state {
                        match state {
                            DecompressState::Mapping => {
                                mapping.insert(current_key.clone(), buf.clone());
                                current_key.clear();
                                buf.clear();
                                break;
                            }
                            DecompressState::Repeat => {
                                res.push_str(&buf.repeat(to_repeat));
                                buf.clear();
                                break;
                            }
                        }
                    } else {
                        to_repeat = buf
                            .parse()
                            .unwrap_or_else(|_| panic!("Error getting repeat number: ({})", buf));

                        buf.clear();
                        state = Some(DecompressState::Repeat);
                    }
                } else {
                    buf.push(chars[i]);
                }

                i += 1;
            }

            state = None;
        } else {
            buf.push(chars[i]);
        }

        i += 1;
    }

    res.push_str(&buf);

    *source = res;

    mapping
}

fn compress(mapping: &mut Mapping, source: &str) -> String {
    let mut res = source.to_owned();

    collapse_repeats(mapping, &mut res);

    loop {
        let used = create_groups(mapping, &mut res);
        println!("{}", mapping.predict_len(res.len()));

        if !used {
            break;
        }
    }

    res
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
        res_str.push(mapping.repeat_char);
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
        let slice = &source[start..end];

        let count = source.match_indices(&slice).collect::<Vec<_>>().len();

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
        let mut start_len = source.len();
        let collapsed = collapse_groups(mapping, source, &group.data);
        let next_key = mapping.next_key();

        let new_occurances = group.occurances - collapsed + 1;

        if collapsed > 0 {
            start_len += 3 + collapsed.to_string().len() + group.data.len();
            start_len -= group.data.len() * (collapsed - 1);
        }

        // 3 because cmd + '=' + cmd
        // next_key.len() * occurances + 1 because the key exists the number of occurances and also
        // when it is defined
        // group.group.len() * occurances - 1 because the data is being replaced all except when it
        // is being defined
        let with_map_len = start_len + 3 + (next_key.len() * (new_occurances + 1))
            - (group.data.len() * (new_occurances - 1));

        if with_map_len < start_len {
            mapping.key_used();
            mapping.insert(next_key.clone(), group.data.clone());
            replace_all(source, &group.data, &next_key);

            return true;
        }
    }

    false
}

fn collapse_groups(mapping: &mut Mapping, source: &mut String, target: &String) -> usize {
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
        replace_val.push(mapping.ctrl_char);
        *source = source.replace(&source[start_index..index], &replace_val)
    }

    count
}

fn replace_all(source: &mut String, target: &String, new_value: &str) {
    while source.contains(target) {
        *source = source.replace(target, new_value);
    }
}

fn find_free_chars(source: &str) -> Vec<char> {
    let possible_chars = r#"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890~`!@#$%^&*()-_+[{}]|;:'"\,<.>/?"#.chars();
    let mut res = vec![];

    for char in possible_chars {
        if !source.contains(char) {
            res.push(char);
        }
    }

    res
}
