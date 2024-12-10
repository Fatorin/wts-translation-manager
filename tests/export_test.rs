#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use wts_translation_manager::data::tooltip::SkillData;
    use wts_translation_manager::utils::common::{FieldType, TextType};
    use wts_translation_manager::utils::export::output_translated;

    #[test]
    fn test_export() {
        let mut map = BTreeMap::new();
        let mut text_type_map = BTreeMap::new();
        text_type_map.insert(FieldType::Researchtip, TextType::SingleLine);
        text_type_map.insert(FieldType::Researchubertip, TextType::MultiLine);
        text_type_map.insert(FieldType::Tip, TextType::SingleLineArray);
        text_type_map.insert(FieldType::Ubertip, TextType::MultiLineArray);

        map.insert(
            "Az02".to_string(),
            SkillData {
                id: "Az02".to_string(),
                researchtip: vec![String::from(r#"|c00ffff80참회는 있지만 후회는 없고|r(W)"#)],
                researchubertip: vec![String::from(
                    r#"|c00ff8080
 ※레벨당 증가치
|c0000ff80 올스텟 1 증가, 지속시간 2초 증가, 페널티 거리 50 감소"#,
                )],
                tip: vec![
                    String::from("|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 1]"),
                    String::from("|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 2]"),
                    String::from("|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 3]"),
                    String::from("|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 4]"),
                    String::from("|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 5]"),
                ],
                ubertip: vec![
                    String::from(
                        "|c009E0ADD지속시간 : |r8초
|c009E0ADD지속효과 : |r올스텟 1 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 400이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r15초",
                    ),
                    String::from(
                        "|c009E0ADD지속시간 : |r10초
|c009E0ADD지속효과 : |r올스텟 2 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 350이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r14초",
                    ),
                    String::from(
                        "|c009E0ADD지속시간 : |r12초
|c009E0ADD지속효과 : |r올스텟 3 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 300이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r13초",
                    ),
                    String::from(
                        "|c009E0ADD지속시간 : |r14초
|c009E0ADD지속효과 : |r올스텟 4 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 250이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r12초",
                    ),
                    String::from(
                        "|c009E0ADD지속시간 : |r16초
|c009E0ADD지속효과 : |r올스텟 5 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 200이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r11초",
                    ),
                ],
                text_type_map,
            },
        );

        match output_translated(&map) {
            Ok(result) => {
                assert_eq!(
                    result,
                    r#"[Az02]
Researchtip = "|c00ffff80참회는 있지만 후회는 없고|r(W)"
Researchubertip = [=[
|c00ff8080
 ※레벨당 증가치
|c0000ff80 올스텟 1 증가, 지속시간 2초 증가, 페널티 거리 50 감소]=]
Tip = {
"|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 1]",
"|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 2]",
"|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 3]",
"|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 4]",
"|c00ffff80참회는 있지만 후회는 없고|r(W) - [레벨 5]",
}
Ubertip = {
[=[
|c009E0ADD지속시간 : |r8초
|c009E0ADD지속효과 : |r올스텟 1 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 400이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r15초]=],
[=[
|c009E0ADD지속시간 : |r10초
|c009E0ADD지속효과 : |r올스텟 2 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 350이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r14초]=],
[=[
|c009E0ADD지속시간 : |r12초
|c009E0ADD지속효과 : |r올스텟 3 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 300이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r13초]=],
[=[
|c009E0ADD지속시간 : |r14초
|c009E0ADD지속효과 : |r올스텟 4 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 250이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r12초]=],
[=[
|c009E0ADD지속시간 : |r16초
|c009E0ADD지속효과 : |r올스텟 5 증가
|c009E0ADD효과 : |r지속 중 충돌 무시, 이동속도 50% 증가, 유체화(잘 보이지 않는다.)
|c009E0ADD패널티 : |r적 서번트와의 거리가 200이하시 모든 효과 자동 해제
|c009E0ADD쿨다운 : |r11초]=],
}

"#
                )
            }
            Err(e) => {
                panic!("{}", e)
            }
        }
    }
}
