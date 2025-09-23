#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug)]
pub struct ResultItem {
    pub text: Option<crate::Text>,
    pub confidence: f32,
    pub bounding_rect: Option<BoundingRect>,
    pub level: PageIteratorLevel,
}

impl BoundingRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Debug)]
pub struct ResultIterator(*mut tesseract_sys::TessResultIterator);

#[derive(Debug)]
pub struct ResultIteratorIter<'a> {
    iterator: &'a mut ResultIterator,
    level: PageIteratorLevel,
    first_iteration: bool,
}

impl Drop for ResultIterator {
    fn drop(&mut self) {
        unsafe { tesseract_sys::TessResultIteratorDelete(self.0) }
    }
}

impl AsRef<*mut tesseract_sys::TessResultIterator> for ResultIterator {
    fn as_ref(&self) -> &*mut tesseract_sys::TessResultIterator {
        &self.0
    }
}

impl ResultIterator {
    pub fn new(raw: *mut tesseract_sys::TessResultIterator) -> Self {
        Self(raw)
    }

    pub fn confidence(&self, level: PageIteratorLevel) -> f32 {
        unsafe { tesseract_sys::TessResultIteratorConfidence(self.0, level.to_sys()) }
    }

    pub fn is_at_beginning_of(&self, level: PageIteratorLevel) -> bool {
        unsafe {
            let page_iter = tesseract_sys::TessResultIteratorGetPageIteratorConst(self.0);
            tesseract_sys::TessPageIteratorIsAtBeginningOf(page_iter, level.to_sys()) != 0
        }
    }

    pub fn is_at_final_element(
        &self,
        level: PageIteratorLevel,
        element: PageIteratorLevel,
    ) -> bool {
        unsafe {
            let page_iter = tesseract_sys::TessResultIteratorGetPageIteratorConst(self.0);
            tesseract_sys::TessPageIteratorIsAtFinalElement(page_iter, level.to_sys(), element.to_sys()) != 0
        }
    }

    pub fn get_bounding_rect(
        &self,
        level: PageIteratorLevel,
    ) -> Option<BoundingRect> {
        unsafe {
            let page_iter = tesseract_sys::TessResultIteratorGetPageIteratorConst(self.0);
            let mut left = 0i32;
            let mut top = 0i32;
            let mut right = 0i32;
            let mut bottom = 0i32;

            let success = tesseract_sys::TessPageIteratorBoundingBox(
                page_iter,
                level.to_sys(),
                &mut left,
                &mut top,
                &mut right,
                &mut bottom,
            );

            if success != 0 {
                Some(BoundingRect {
                    left,
                    top,
                    right,
                    bottom,
                })
            } else {
                None
            }
        }
    }

    pub fn next(&mut self, level: PageIteratorLevel) -> bool {
        unsafe { tesseract_sys::TessResultIteratorNext(self.0, level.to_sys()) != 0 }
    }

    pub fn get_utf8_text(
        &self,
        level: PageIteratorLevel,
    ) -> Option<crate::Text> {
        unsafe {
            let text_ptr = tesseract_sys::TessResultIteratorGetUTF8Text(self.0, level.to_sys());
            if text_ptr.is_null() {
                None
            } else {
                Some(crate::Text::new(text_ptr))
            }
        }
    }

    pub fn iter_at_level(
        &mut self,
        level: PageIteratorLevel,
    ) -> ResultIteratorIter<'_> {
        ResultIteratorIter {
            iterator: self,
            level,
            first_iteration: true,
        }
    }

    pub fn words(&mut self) -> ResultIteratorIter<'_> {
        self.iter_at_level(PageIteratorLevel::Word)
    }

    pub fn symbols(&mut self) -> ResultIteratorIter<'_> {
        self.iter_at_level(PageIteratorLevel::Symbol)
    }

    pub fn lines(&mut self) -> ResultIteratorIter<'_> {
        self.iter_at_level(PageIteratorLevel::Textline)
    }

    pub fn paragraphs(&mut self) -> ResultIteratorIter<'_> {
        self.iter_at_level(PageIteratorLevel::Para)
    }

    pub fn blocks(&mut self) -> ResultIteratorIter<'_> {
        self.iter_at_level(PageIteratorLevel::Block)
    }
}

impl<'a> ResultIteratorIter<'a> {
    pub fn is_at_beginning_of(&self, level: PageIteratorLevel) -> bool {
        self.iterator.is_at_beginning_of(level)
    }

    pub fn is_at_final_element(
        &self,
        level: PageIteratorLevel,
        element: PageIteratorLevel,
    ) -> bool {
        self.iterator.is_at_final_element(level, element)
    }

    pub fn get_bounding_rect(
        &self,
        level: PageIteratorLevel,
    ) -> Option<BoundingRect> {
        self.iterator.get_bounding_rect(level)
    }

    pub fn confidence(&self, level: PageIteratorLevel) -> f32 {
        self.iterator.confidence(level)
    }

    pub fn get_utf8_text(
        &self,
        level: PageIteratorLevel,
    ) -> Option<crate::Text> {
        self.iterator.get_utf8_text(level)
    }
}

impl<'a> Iterator for ResultIteratorIter<'a> {
    type Item = ResultItem;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first_iteration {
            if !self.iterator.next(self.level) {
                return None;
            }
        } else {
            self.first_iteration = false;
        }

        let text = self.iterator.get_utf8_text(self.level);
        let confidence = self.iterator.confidence(self.level);
        let bounding_rect = self.iterator.get_bounding_rect(self.level);

        Some(ResultItem {
            text,
            confidence,
            bounding_rect,
            level: self.level,
        })
    }
}
