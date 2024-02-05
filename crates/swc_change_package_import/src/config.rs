use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Config {
  /// 配置：
  /// ```rs
  /// Config::LiteralConfig(String::from("antd"))
  /// ```
  /// 效果：
  /// ```js
  /// import { Button } from "antd";
  /// // --->
  /// import Button from "antd/Button";
  /// ```
  LiteralConfig(String),
  /// 配置：
  /// ```rs
  /// Config::SpecificConfig(
  //      SpecificConfigs {
  //          name: String::from("ice"),
  //          map: HashMap::from([
  //              (
  //                  "a".to_string(),
  //                  MapProperty {
  //                      to: String::from("@ice/x/y"),
  //                      import_type: None,
  //                      name: None,
  //                  }
  //               ),
  //          ]),
  //      }
  //  ),
  /// ```
  /// 效果：
  /// ```js
  /// import { a } from "ice";
  /// // --->
  /// import a from "@ice/x/y";
  /// ```
  /// 
  /// 更多配置请参考[文档](https://alidocs.dingtalk.com/i/nodes/20eMKjyp810mMdK4Ho1LpqX7JxAZB1Gv?utm_scene=team_space)
  SpecificConfig(SpecificConfigs),
}

#[derive(Debug, Clone ,Serialize, Deserialize)]
pub struct SpecificConfigs {
  pub name: String,
  pub map: HashMap<String, MapProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapProperty {
  pub to: String,
  pub import_type: Option<ImportType>,
  pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportType {
  Named,
  Default,
}
