use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    pub msg: Option<String>,       // 空字符串 ""
    pub code: u8,                  // 数字 0
    pub data: Option<Vec<Course>>, // 数据数组
    pub count: Option<u8>,         // 总数
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Course {
    pub kch: Option<String>,      // 课程号
    pub kc_mc: Option<String>,    // 课程名称
    pub jg0101mc: Option<String>, // 教师名称
    pub jsgh: Option<String>,     // 教师工号
    pub kt_mc: Option<String>,    // 课堂名称
    pub pkrs: Option<u16>,        // 排课人数
    pub xkrs: Option<u16>,        // 选课人数
    pub kcxz: Option<String>,     // 课程性质
    pub kclb: Option<String>,     // 课程类别
    pub jx0404id: Option<String>, // 教学ID
    pub fzmc: Option<String>,     // 分组名称
    pub sktime: Option<String>,   // 上课时间
    pub skddmc: Option<String>,   // 上课地点
    pub skxqmc: Option<String>,   // 校区
    pub kkyx: Option<String>,     // 开课院系
    pub zhouxs: Option<String>,   // 周学时
    pub xf: Option<f32>,          // 学分
    pub zxs: Option<u8>,          // 总学时
    pub khfs: Option<String>,     // 考核方式
    pub xsfl0: Option<u8>,        // 理论学时
    pub xsfl1: Option<u8>,        // 实验学时
    pub xkh: Option<String>,      // 选课号
    pub bj: Option<String>,       // 备注
    pub rownum_: Option<u8>,      // 行号
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semester {
    pub id: String,   // e.g., "2025-2026-2"
    pub name: String, // e.g., "2025秋季学期"
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct CourseMeta {
    pub semesters: Vec<Semester>,
    pub kbjcmsid: String, // 动态提取的 ID
}
