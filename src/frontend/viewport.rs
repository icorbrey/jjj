use std::ops::Range;

#[tracing::instrument]
pub fn compute_sliding_window(
    item_length: usize,
    selection: usize,
    viewport_y: usize,
    viewport_height: usize,
    viewport_margin: usize,
) -> (usize, Range<usize>) {
    let computed_y = if item_length < viewport_height {
        0
    } else if selection < viewport_y + viewport_margin {
        selection.saturating_sub(viewport_margin)
    } else if viewport_y + viewport_height - viewport_margin - 1 < selection {
        selection + viewport_margin - viewport_height + 1
    } else {
        viewport_y
    };

    tracing::trace!(
        computed_y,
        "{:?}",
        computed_y..usize::min(item_length, computed_y + viewport_height),
    );

    (
        computed_y,
        computed_y..usize::min(item_length, computed_y + viewport_height),
    )
}
