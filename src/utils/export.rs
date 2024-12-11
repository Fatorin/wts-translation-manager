use crate::data::tooltip::{SkillData, TooltipData};
use crate::utils::common::*;
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::iter::Peekable;

pub fn export_files(data: &TooltipData) -> Result<(), String> {
    let output = output_files(&data.skill_manager.translation_skills)?;
    fs::write(EXPORT_FILE_NAME, output).expect("Unable to write file");
    Ok(())
}

pub fn output_files(translation_skills: &BTreeMap<String, SkillData>) -> Result<String, String> {
    let content = fs::read_to_string(SOURCE_FILE_NAME).expect("Unable to read file");
    let mut lines = content.lines().peekable();
    let mut current_id = String::new();
    let mut output = String::new();

    while let Some(line) = lines.next() {
        if let Some(id) = get_id(line) {
            current_id = id;
        }

        match get_field_type(line) {
            Some(field_type) => match translation_skills.get(&current_id) {
                Some(data) => {
                    if let Some(text_type) = data.text_type_map.get(&field_type) {
                        let source_text_type = get_text_type(line, &mut lines)
                            .ok_or_else(|| "Not found text type form source skill")?;
                        skip_lines(&source_text_type, &mut lines);
                        let field_value = get_field_value(&field_type, text_type, data)?;
                        output.push_line(&field_value);
                    } else {
                        output.push_line(line);
                    }
                }
                None => {
                    output.push_line(line);
                }
            },
            None => {
                output.push_line(line);
            }
        }
    }

    Ok(output)
}

fn skip_lines<'a, I>(text_type: &TextType, lines: &mut Peekable<I>)
where
    I: Iterator<Item = &'a str>,
{
    match text_type {
        TextType::SingleLine => {}
        TextType::MultiLine => {
            while let Some(line) = lines.next() {
                if line.ends_with("]=]") {
                    break;
                }
            }
        }
        TextType::SingleLineArray
        | TextType::SingleLineArrayExt
        | TextType::MultiLineArray
        | TextType::MultiLineArrayExt => {
            while let Some(line) = lines.next() {
                println!("FOUND LINE: {}", line);
                if line.contains("}") {
                    break;
                }
            }
        }
    }
}

fn get_field_value(
    field_type: &FieldType,
    text_type: &TextType,
    data: &SkillData,
) -> Result<String, String> {
    let field_value = match field_type {
        FieldType::Researchtip => output_field_value(&field_type, text_type, &data.researchtip),
        FieldType::Researchubertip => {
            output_field_value(&field_type, text_type, &data.researchubertip)
        }
        FieldType::Tip => output_field_value(&field_type, text_type, &data.tip),
        FieldType::Ubertip => output_field_value(&field_type, text_type, &data.ubertip),
    }?;
    Ok(field_value)
}

pub fn export_translated(data: &TooltipData) -> Result<(), String> {
    let output = output_translated(&data.skill_manager.translation_skills)?;
    fs::write(TRANSLATE_FILE_NAME, output).map_err(|e| e.to_string())
}

pub fn output_translated(skills: &BTreeMap<String, SkillData>) -> Result<String, String> {
    let mut output = String::new();
    for (key, data) in skills {
        let field_name = output_field_name(key)?;
        output.push_line(field_name.as_str());

        for (field_type, text_type) in &data.text_type_map {
            let field_value = match field_type {
                FieldType::Researchtip => {
                    output_field_value(field_type, text_type, &data.researchtip)
                }
                FieldType::Researchubertip => {
                    output_field_value(field_type, text_type, &data.researchubertip)
                }
                FieldType::Tip => output_field_value(field_type, text_type, &data.tip),
                FieldType::Ubertip => output_field_value(field_type, text_type, &data.ubertip),
            }?;

            if !field_value.is_empty() {
                output.push_line(field_value.as_str());
            }
        }
        output.push(NEWLINE_SYMBOL);
    }

    Ok(output.to_string())
}

fn output_field_name(key: &String) -> Result<String, String> {
    let pattern = Regex::new(EXPORT_ID_REGEX).unwrap();
    if let Some(caps) = pattern.captures(key) {
        if caps.get(1).is_some() {
            return Ok(format!("[{}]", caps.get(1).unwrap().as_str()));
        }
        if caps.get(2).is_some() {
            return Ok(format!("[\"{}\"]", caps.get(2).unwrap().as_str()));
        }
    }
    Err(format!("Unable to parse key :{}", key))
}

fn output_field_value(
    field_type: &FieldType,
    text_type: &TextType,
    value: &Vec<String>,
) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }

    let field_name = field_type.to_str();
    let result = match text_type {
        TextType::SingleLine | TextType::MultiLine => {
            output_single_or_multi_line(field_name, value)
        }
        TextType::SingleLineArray => output_single_line_array(field_name, value),
        TextType::SingleLineArrayExt => output_single_line_array_ext(field_name, value),
        TextType::MultiLineArray => output_multi_line_array(field_name, value),
        TextType::MultiLineArrayExt => output_multi_line_array_ext(field_name, value),
    };

    result
}

fn output_single_or_multi_line(field_name: &str, value: &Vec<String>) -> Result<String, String> {
    let mut result = String::new();

    if value.len() != 1 {
        return Err("singleline不合法".to_string());
    }

    if value[0].contains('\n') {
        result.push_line(format!("{} = [=[", field_name).as_str());
        result.push_str(format!("{}]=]", value[0]).as_str());
    } else {
        result.push_str(format!("{} = \"{}\"", field_name, value[0]).as_str());
    }
    Ok(result)
}

fn output_single_line_array(field_name: &str, value: &Vec<String>) -> Result<String, String> {
    let mut result = String::new();
    result.push_line(format!("{} = {{", field_name).as_str());
    for str in value {
        result.push_line(format!("\"{}\",", str).as_str())
    }
    result.push_str("}");
    Ok(result)
}

fn output_single_line_array_ext(field_name: &str, value: &Vec<String>) -> Result<String, String> {
    let mut result = String::new();
    result.push_line(format!("{} = {{", field_name).as_str());
    for (index, str) in value.iter().enumerate() {
        result.push_line(format!("{} = \"{}\",", index + 1, str).as_str())
    }
    result.push_str("}");
    Ok(result)
}

fn output_multi_line_array(field_name: &str, value: &Vec<String>) -> Result<String, String> {
    let mut result = String::new();
    result.push_line(format!("{} = {{", field_name).as_str());

    for str in value {
        result.push_line("[=[");
        result.push_line(format!("{}]=],", str).as_str());
    }

    result.push_str("}");
    Ok(result)
}

fn output_multi_line_array_ext(field_name: &str, value: &Vec<String>) -> Result<String, String> {
    let mut result = String::new();
    result.push_line(format!("{} = {{", field_name).as_str());

    for (index, str) in value.iter().enumerate() {
        result.push_line(format!("{} = [=[", index + 1).as_str());
        result.push_line(format!("{}]=],", str).as_str());
    }

    result.push_str("}");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_single_line() {
        let field_name = FieldType::Tip.to_str();
        let sample = String::from("마력 전달과 흡수|cffffcc00(Q)|r ");
        let value = vec![sample];
        if let Ok(result) = output_single_or_multi_line(field_name, &value) {
            assert!(!result.is_empty());
            assert_eq!(result, r#"Tip = "마력 전달과 흡수|cffffcc00(Q)|r ""#);
        } else {
            panic!("Something went wrong");
        }
    }

    #[test]
    fn test_output_multi_line() {
        let field_name = FieldType::Researchubertip.to_str();
        let sample = String::from(
            r#"|c00ff8080
 ※레벨당 능력
|c0000ff80직선상 적에게 엄청난 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 1|r - 정면 1600범위에 1000의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 2|r - 정면 1600범위에 1350의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 3|r - 정면 1600범위에 1700의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 4|r - 정면 1600범위에 2050의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 5|r - 정면 1600범위에 2400의 데미지를 입히는 붉은 회오리를 방출합니다."#,
        );
        let value = vec![sample];
        if let Ok(result) = output_single_or_multi_line(field_name, &value) {
            assert!(!result.is_empty());
            assert_eq!(
                result,
                r#"Researchubertip = [=[
|c00ff8080
 ※레벨당 능력
|c0000ff80직선상 적에게 엄청난 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 1|r - 정면 1600범위에 1000의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 2|r - 정면 1600범위에 1350의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 3|r - 정면 1600범위에 1700의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 4|r - 정면 1600범위에 2050의 데미지를 입히는 붉은 회오리를 방출합니다.
 |cffffcc00레벨 5|r - 정면 1600범위에 2400의 데미지를 입히는 붉은 회오리를 방출합니다.]=]"#
            );
        } else {
            panic!("Something went wrong");
        }
    }

    #[test]
    fn test_output_single_line_array() {
        let field_name = FieldType::Tip.to_str();
        let value = vec![
            "|c00ffff80무공|cffffcc00(A1)|r",
            "|c00ffff80무공|cffffcc00(A2)|r",
        ];
        let value: Vec<String> = value.iter().map(|s| s.to_string()).collect();
        if let Ok(result) = output_single_line_array(field_name, &value) {
            assert!(!result.is_empty());
            assert_eq!(
                result,
                r#"Tip = {
"|c00ffff80무공|cffffcc00(A1)|r",
"|c00ffff80무공|cffffcc00(A2)|r",
}"#
            );
        } else {
            panic!("Something went wrong");
        }
    }

    #[test]
    fn test_output_single_line_array_ext() {
        let field_name = FieldType::Tip.to_str();
        let value = vec![
            "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 1|r]",
            "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 2|r]",
        ];
        let value: Vec<String> = value.iter().map(|s| s.to_string()).collect();
        if let Ok(result) = output_single_line_array_ext(field_name, &value) {
            assert!(!result.is_empty());
            assert_eq!(
                result,
                r#"Tip = {
1 = "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 1|r]",
2 = "|c00ffff80천지를 가르는 개벽의 별 [Enuma Elish]|r(|cffffcc00R|r) - |r[|cffffcc00레벨 2|r]",
}"#
            );
        } else {
            panic!("Not a multi line");
        }
    }

    #[test]
    fn test_output_multi_line_array() {
        let field_name = FieldType::Ubertip.to_str();
        let value = vec![
            r#"|c0080ffff속성 습득 포인트: 12

|c009E0ADD절대쿨다운|r : 120초(힘/공투자 중 더 높은쪽 적용, 투자 당 1초씩 감소)"#,
            r#"|c0080ffff속성 습득 포인트: 8

|c00ff8080노템모드 패널티|r : 반사데미지50% 감소,대마력증가량 5%감소"#,
        ];
        let value: Vec<String> = value.iter().map(|s| s.to_string()).collect();
        if let Ok(result) = output_multi_line_array(field_name, &value) {
            assert!(!result.is_empty());
            assert_eq!(
                result,
                r#"Ubertip = {
[=[
|c0080ffff속성 습득 포인트: 12

|c009E0ADD절대쿨다운|r : 120초(힘/공투자 중 더 높은쪽 적용, 투자 당 1초씩 감소)]=],
[=[
|c0080ffff속성 습득 포인트: 8

|c00ff8080노템모드 패널티|r : 반사데미지50% 감소,대마력증가량 5%감소]=],
}"#
            );
        } else {
            panic!("Not a multi line");
        }
    }

    #[test]
    fn test_output_multi_line_array_ext() {
        let field_name = FieldType::Ubertip.to_str();
        let value = vec![
            r#"|c009E0ADD데미지|r : 1000

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다."#,
            r#"|c009E0ADD데미지|r : 1350

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다."#,
        ];
        let value: Vec<String> = value.iter().map(|s| s.to_string()).collect();
        if let Ok(result) = output_multi_line_array_ext(field_name, &value) {
            assert!(!result.is_empty());
            assert_eq!(
                result,
                r#"Ubertip = {
1 = [=[
|c009E0ADD데미지|r : 1000

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다.]=],
2 = [=[
|c009E0ADD데미지|r : 1350

괴리검 에아에 의한 공간 절단. 압축되어서 서로 마찰하는 풍압의 단층은, 의사적인 시공단층이 되어 적대하는 모든 것을 분쇄합니다.]=],
}"#
            );
        } else {
            panic!("Not a multi line");
        }
    }
}
