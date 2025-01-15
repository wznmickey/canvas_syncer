use scraper::{Html, Selector};

pub fn get_file_link_from_html(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("a[data-api-endpoint]").unwrap();
    let values: Vec<String> = document
        .select(&selector)
        .filter_map(|element| element.value().attr("data-api-endpoint").map(String::from))
        .collect();
    values
}
