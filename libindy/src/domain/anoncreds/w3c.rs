use indy_api_types::errors::{IndyResult, IndyErrorKind, IndyResultExt};
use super::proof::Proof;

#[derive(Debug, Serialize, Deserialize)]
struct W3cProof {
    #[serde(rename = "type")]
    typ: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DerivedCredential {
    #[serde(rename = "@context")]
    context: Vec<String>,
    #[serde(rename = "type")]
    typ: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifiablePresentation {
    #[serde(rename = "@context")]
    context: Vec<String>,
    #[serde(rename = "type")]
    typ: String,
    #[serde(rename = "verifiableCredential")]
    creds: Vec<DerivedCredential>,
    proof: W3cProof,
}

#[allow(dead_code)]
pub fn to_vp(proof: &Proof) -> IndyResult<String> {
    let preso = VerifiablePresentation {
        context: vec![
            "https://www.w3.org/2018/credentials/v1".to_string(),
            proof.identifiers[0].cred_def_id.0.to_string()
        ],
        typ: "VerifiablePresentation".to_string(),
        creds: vec![DerivedCredential {
            context: vec![
                "https://www.w3.org/2018/credentials/v1".to_string(),
            ],
            typ: vec!["VerifiableCredential".to_string()]
        }],
        proof: W3cProof { typ: "AnonCredPresentationProofv1".to_string() },
    };
    serde_json::to_string(&preso)
        .to_indy(IndyErrorKind::InvalidState, "Cannot serialize FullProof")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn mapping_works() {
        let proof: Proof = serde_json::from_str(FAKE_PROOF_JSON).unwrap();
        let vp = to_vp(&proof).unwrap();
        let mut errors: Vec<String> = Vec::new();
        let v: Value = serde_json::from_str(&vp).unwrap();

        check_structure(&v, "@context", "HAS https://www.w3.org/2018/credentials/v1", &mut errors);
        check_structure(&v, "type", "LIKE VerifiablePresentation", &mut errors);
        if check_structure(&v, "verifiableCredential", "is array", &mut errors) {
            let vcs = v["verifiableCredential"].as_array().unwrap();
            let mut i: usize = 0;
            for vc in vcs {
                check_vc(&vc, i, &mut errors);
                i += 1;
            }
        }
        if check_structure(&v, "proof", "is object", &mut errors) {

        }

        if !errors.is_empty() {
            panic!("Presentation structure has errors: {}.\n\nPresentation was: {}",
                   &errors.join(". "), &vp);
        }
    }


    fn array_has_value(candidate: &Value, value: &str) -> bool {
        if candidate.is_array() {
            let ar = candidate.as_array().unwrap();
            for i in 0..ar.len() {
                let item = ar[i].to_string();
                // Ignore the delimiting quotes around str value. Compare inner only.
                let mut txt = item.as_str();
                let bytes = txt.as_bytes();
                if bytes.len() >= 2 && (bytes[0] == b'"') && (bytes[bytes.len() - 1] == b'"') {
                    txt = &item.as_str()[1..item.len() - 1];
                }
                if txt.eq(value) {
                    return true;
                }
            }
        }
        false
    }

    fn text_matches_regex(candidate: &Value, regex: &str) -> bool {
        if candidate.is_string() {
            use regex::Regex;
            let pat = Regex::new(regex).unwrap();
            if pat.is_match(candidate.as_str().unwrap()) {
                return true;
            }
        }
        false
    }

    fn check_structure(container: &Value, path: &str, expected: &str, errors: &mut Vec<String>) -> bool {
        let mut ok = false;
        let i = path.rfind('/');
        let subitem = if i.is_some() { &path[i.unwrap() + 1..] } else { &path[..] };
        let item = &container[subitem];
        if !item.is_null() {
            match expected {
                "is array" => ok = item.is_array(),
                "is object" => ok = item.is_object(),
                "is number" => ok = item.is_number(),
                "is string" => ok = item.is_string(),
                _ => {
                    if expected[0..4].eq("HAS ") {
                        ok = array_has_value(item, &expected[4..]);
                    } else if expected[0..5].eq("LIKE ") {
                        ok = text_matches_regex(item, &expected[5..]);
                    }
                }
            }
        }
        if !ok {
            errors.push(format!("Expected {} {}", path.to_string(), expected));
        }
        ok
    }

    fn check_vc(vc: &Value, i: usize, errors: &mut Vec<String>) {
        let prefix = format!("verifiableCredential[{}]", i);
        check_structure(&vc, format!("{}/type", &prefix).as_str(), "HAS VerifiableCredential", errors);
        check_structure(&vc, format!("{}/@context", &prefix).as_str(), "HAS https://www.w3.org/2018/credentials/v1", errors);
    }

    // This JSON exhibits the actual structure of a proof, but numeric values
    // are wrong and strings have been shortened. Thus, it should deserialize
    // correctly but will not validate.
    const FAKE_PROOF_JSON: &'static str = r#"{
  "proof":{
    "proofs":[
      {
        "primary_proof":{
          "eq_proof":{
            "revealed_attrs":{
              "height":"175",
              "name":"1139481716457488690172217916278103335"
            },
            "a_prime":"5817705...096889",
            "e":"1270938...756380",
            "v":"1138...39984052",
            "m":{
              "master_secret":"375275...0939395",
              "sex":"3511483...897083518",
              "age":"13430...63372249"
            },
            "m2":"1444497...2278453"
          },
          "ge_proofs":[
            {
              "u":{
                "1":"152500...3999140",
                "2":"147748...2005753",
                "0":"8806...77968",
                "3":"10403...8538260"
              },
              "r":{
                "2":"15706...781609",
                "3":"343...4378642",
                "0":"59003...702140",
                "DELTA":"9607...28201020",
                "1":"180097...96766"
              },
              "mj":"134300...249",
              "alpha":"827896...52261",
              "t":{
                "2":"7132...47794",
                "3":"38051...27372",
                "DELTA":"68025...508719",
                "1":"32924...41082",
                "0":"74906...07857"
              },
              "predicate":{
                "attr_name":"age",
                "p_type":"GE",
                "value":18
              }
            }
          ]
        },
        "non_revoc_proof":null
      }
    ],
    "aggregated_proof":{
      "c_hash":"108743...92564",
      "c_list":[
        [0,1,2,3,4,255],
        [0,1,2,3,4,255],
        [0,1,2,3,4,255],
        [0,1,2,3,4,255],
        [0,1,2,3,4,255],
        [0,1,2,3,4,255]
      ]
    }
  },
  "requested_proof":{
    "revealed_attrs":{
      "attr1_referent":{
        "sub_proof_index":0,
        "raw":"Alex",
        "encoded":"1139481716457488690172217916278103335"
      }
    },
    "revealed_attr_groups":{
      "attr4_referent":{
        "sub_proof_index":0,
        "values":{
          "name":{
            "raw":"Alex",
            "encoded":"1139481716457488690172217916278103335"
          },
          "height":{
            "raw":"175",
            "encoded":"175"
          }
        }
      }
    },
    "self_attested_attrs":{
      "attr3_referent":"8-800-300"
    },
    "unrevealed_attrs":{
      "attr2_referent":{
        "sub_proof_index":0
      }
    },
    "predicates":{
      "predicate1_referent":{
        "sub_proof_index":0
      }
    }
  },
  "identifiers":[
    {
      "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
      "cred_def_id":"NcYxi...cYDi1e:2:gvt:1.0:TAG_1",
      "rev_reg_id":null,
      "timestamp":null
    }
  ]
}"#;

}
