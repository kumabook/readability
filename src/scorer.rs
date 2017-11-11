use std::rc::Rc;
use std::path::Path;
use std::cell::Cell;
use std::collections::BTreeMap;
use url::Url;
use regex::Regex;
use html5ever::tree_builder::TreeSink;
use html5ever::rcdom::Node;
use html5ever::rcdom::NodeData::{Element, Text};
use html5ever::rcdom::Handle;
use html5ever::rcdom::NodeData::{
    Document,
    Doctype,
    Comment,
    ProcessingInstruction
};
use html5ever::rcdom::RcDom;
use dom;

pub static PUNCTUATIONS: &'static str = r"([、。，．！？]|\.[^A-Za-z0-9]|,[^0-9]|!|\?)";
pub static POSITIVE: &'static str = "article|body|content|entry|hentry|main|page\
                                     |pagination|post|text|blog|story";
pub static NEGATIVE: &'static str = "combx|comment|com|contact|foot|footer|footnote\
                                     |masthead|media|meta|outbrain|promo|related\
                                     |scroll|shoutbox|sidebar|sponsor|shopping\
                                     |tags|tool|widget|form|textfield";
static BLOCK_CHILD_TAGS: [&'static str; 10] = [
    "a", "blockquote", "dl", "div", "img", "ol", "p", "pre", "table", "ul",
];

pub struct Candidate {
    pub node:  Rc<Node>,
    pub score: Cell<f32>,
}

pub fn fix_img_path(handle: Handle, url: &Url) -> bool {
    let src = dom::get_attr("src", handle.clone());
    if src.is_none() {
        return false
    }
    let s = src.unwrap();
    if !s.starts_with("//") && !s.starts_with("http://") && s.starts_with("https://") {
        match url.join(&s) {
            Ok(new_url) => dom::set_attr("src", new_url.as_str(), handle),
            Err(_)      => (),
        }
    }
    true
}

pub fn get_link_density(handle: Handle) -> f32 {
    let text_length = dom::text_len(handle.clone()) as f32;
    if text_length == 0.0 {
        return 0.0;
    }
    let mut link_length = 0.0;
    let mut links: Vec<Rc<Node>> = vec![];
    dom::find_node(handle.clone(), "a", &mut links);
    for link in links.iter() {
        link_length += dom::text_len(link.clone()) as f32;
    }
    link_length / text_length
}

pub fn is_candidate(handle: Handle) -> bool {
    let text_len = dom::text_len(handle.clone());
    if text_len < 20 {
        return false
    }
    let n: &str = &dom::get_tag_name(handle. clone()).unwrap_or("".to_string());
    match n {
        "p" => true,
        "div" | "article" | "center" | "section" =>
            !dom::has_nodes(handle.clone(), &BLOCK_CHILD_TAGS.iter().map(|t| *t).collect()),
        _ => false
    }
}

pub fn init_content_score(handle: Handle) -> f32 {
    let tag_name = dom::get_tag_name(handle.clone()).unwrap_or("".to_string());
    match tag_name.as_ref() {
        "article"    => 10.0,
        "div"        => 5.0,
        "blockquote" => 3.0,
        "form"       => -3.0,
        "th"         => 5.0,
        _            => 0.0,
    }
}

pub fn calc_content_score(handle: Handle) -> f32 {
    let mut score: f32 = 1.0;
    score += get_class_weight(handle.clone());
    let mut text = String::new();
    dom::extract_text(handle.clone(), &mut text, true);
    let re = Regex::new(PUNCTUATIONS).unwrap();
    let mat = re.find_iter(&text);
    score += mat.count() as f32;
    score += f32::min(f32::floor(text.chars().count() as f32 / 100.0), 3.0);
    return score
}

pub fn get_class_weight(handle: Handle) -> f32 {
    let mut weight: f32 = 0.0;
    match handle.data {
        Element { name: _, ref attrs, .. } => {
            for prop in ["id", "class"].iter() {
                if let Some(class) = dom::attr(prop, &attrs.borrow()) {
                    if Regex::new(POSITIVE).unwrap().is_match(&class) {
                        weight += 25.0
                    };
                    if Regex::new(NEGATIVE).unwrap().is_match(&class) {
                        weight -= 25.0
                    }
                }
            }
        },
        _ => (),
    };
    weight
}

pub fn find_candidates(mut dom:    &mut RcDom,
                       id:         &Path,
                       handle:     Handle,
                       candidates: &mut BTreeMap<String, Candidate>,
                       nodes:      &mut BTreeMap<String, Rc<Node>>) {

    if let Some(id) = id.to_str().map(|id| id.to_string()) {
        nodes.insert(id, handle.clone());
    }

    if is_candidate(handle.clone()) {
        let score = calc_content_score(handle.clone());
        if let Some(c) = id.parent()
            .and_then(|id| find_or_create_candidate(id, candidates, nodes))
        {
            c.score.set(c.score.get() + score)
        }
        if let Some(c) = id.parent()
            .and_then(|id| id.parent())
            .and_then(|id| find_or_create_candidate(id, candidates, nodes))
        {
            c.score.set(c.score.get() + score / 2.0)
        }
    }


    if is_candidate(handle.clone()) {
        let score = calc_content_score(handle.clone());
        if let Some(c) = id.to_str()
            .map(|id| id.to_string())
            .and_then(|id| candidates.get(&id)) {
                c.score.set(c.score.get() + score)
            }
        if let Some(c) = id.parent()
            .and_then(|pid| pid.to_str())
            .map(|id| id.to_string())
            .and_then(|pid| candidates.get(&pid)) {
                c.score.set(c.score.get() + score)
            }
        if let Some(c) = id.parent()
            .and_then(|p| p.parent())
            .and_then(|pid| pid.to_str())
            .map(|id| id.to_string())
            .and_then(|pid| candidates.get(&pid)) {
                c.score.set(c.score.get() + score)
            }
    }

    for (i, child) in handle.children.borrow().iter().enumerate() {
        find_candidates(&mut dom,
                        id.join(i.to_string()).as_path(),
                        child.clone(),
                        candidates,
                        nodes)
    }
}

fn find_or_create_candidate<'a>(id: &Path,
                                candidates: &'a mut BTreeMap<String, Candidate>,
                                nodes: &BTreeMap<String, Rc<Node>>) -> Option<&'a Candidate> {
    if let Some(id) = id.parent()
        .and_then(|pid| pid.to_str())
        .map(|id| id.to_string())
    {
        if let Some(node) = nodes.get(&id) {
            if candidates.get(&id).is_none() {
                candidates.insert(id.clone(), Candidate {
                    node:  node.clone(),
                    score: Cell::new(init_content_score(node.clone())),
                });
            }
            return candidates.get(&id)
        }
    }
    None
}

pub fn clean(mut dom: &mut RcDom, id: &Path, handle: Handle, url: &Url, candidates: &BTreeMap<String, Candidate>) -> bool {
    let mut useless = false;
    match handle.data {
        Document       => (),
        Doctype { .. } => (),
        Text { ref contents } => {
            let s = contents.borrow();
            if s.trim().len() == 0 {
                useless = true
            }
        },
        Comment { .. } => useless = true,
        Element { ref name, ref attrs, .. } => {
            let tag_name = name.local.as_ref();
            match tag_name.to_lowercase().as_ref() {
                "script" | "link" | "style" | "noscript" | "meta"
                    | "h1" | "object" | "header" | "footer" | "aside" => {
                    useless = true
                },
                "form" | "table" | "ul" | "div" => {
                    useless = is_useless(id, handle.clone(), candidates)
                },
                "img" => useless = fix_img_path(handle.clone(), url),
                _     => (),
            }
            dom::clean_attr("id"   , &mut *attrs.borrow_mut());
            dom::clean_attr("class", &mut *attrs.borrow_mut());
            dom::clean_attr("style", &mut *attrs.borrow_mut());
        },
        ProcessingInstruction { .. } => unreachable!()
    }
    let mut useless_nodes = vec![];
    for (i, child) in handle.children.borrow().iter().enumerate() {
        let pid = id.join(i.to_string());
        if clean(&mut dom, pid.as_path(), child.clone(), url, candidates) {
            useless_nodes.push(child.clone());
        }
    }
    for node in useless_nodes.iter() {
        dom.remove_from_parent(node);
    }
    if dom::is_empty(handle) {
        useless = true
    }
    useless
}

pub fn is_useless(id: &Path, handle: Handle, candidates: &BTreeMap<String, Candidate>) -> bool {
    let tag_name = &dom::get_tag_name(handle.clone()).unwrap_or("".to_string());
    let weight = get_class_weight(handle.clone());
    let score = id.to_str()
        .and_then(|id| candidates.get(id))
        .map(|c| c.score.get()).unwrap_or(0.0);
    if weight + score < 0.0 {
        return true
    }
    let mut p_nodes:     Vec<Rc<Node>> = vec![];
    let mut img_nodes:   Vec<Rc<Node>> = vec![];
    let mut li_nodes:    Vec<Rc<Node>> = vec![];
    let mut input_nodes: Vec<Rc<Node>> = vec![];
    let mut embed_nodes: Vec<Rc<Node>> = vec![];
    dom::find_node(handle.clone(), "p"     , &mut p_nodes);
    dom::find_node(handle.clone(), "img"   , &mut img_nodes);
    dom::find_node(handle.clone(), "li"    , &mut li_nodes);
    dom::find_node(handle.clone(), "input" , &mut input_nodes);
    dom::find_node(handle.clone(), "embed" , &mut embed_nodes);
    let p_count        = p_nodes.len();
    let img_count      = img_nodes.len();
    let li_count       = li_nodes.len() as i32 - 100;
    let input_count    = input_nodes.len();
    let embed_count    = embed_nodes.len();
    let link_density   = get_link_density(handle.clone());
    let content_length = dom::text_len(handle.clone());

    if img_count > p_count {
        return true
    }
    if li_count > p_count as i32 && tag_name != "ul" && tag_name != "ol" {
        return true
    }
    if input_count as f32 > f32::floor(p_count as f32 / 3.0) {
        return true
    }
    if content_length < 25 && (img_count == 0 || img_count > 2) {
        return true
    }
    if weight < 25.0 && link_density > 0.2 {
        return true
    }
    if (embed_count == 1 && content_length < 35) || embed_count > 1 {
        return true
    }
    return false
}
