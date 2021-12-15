
/**
This mod focuses on comparing the checksums of the webpages.
This is only effective for static webpages, however since static webpages are no longer the norm
We only use this as a very basic (but very fast) technique to filter out static webpage attacks
(If the checksum is equal then there's no way the page was defaced)
*/

/*
Compare the fully rendered dom
 */
pub fn comp_doms(initial_dom: &str, current_dom: &str) -> bool {

    let mut digestor = sha1::Sha1::new();
    let mut digestor_current = sha1::Sha1::new();

    digestor.update(initial_dom.as_bytes());
    digestor_current.update(current_dom.as_bytes());

    digestor.digest() == digestor_current.digest()
}

//TODO: should we compare the pdfs and maybe screenshots?