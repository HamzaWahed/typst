use crate::layout::{AlignNode, GridNode, Sizing, TrackSizings};
use crate::prelude::*;

/// # Table
/// A table of items.
///
/// Tables are used to arrange content in cells. Cells can contain arbitrary
/// content, including multiple paragraphs and are specified in row-major order.
/// Because tables are just grids with configurable cell properties, refer to
/// the [grid documentation]($func/grid) for more information on how to size the
/// table tracks.
///
/// ## Example
/// ```example
/// #table(
///   columns: (1fr, auto, auto),
///   inset: 10pt,
///   align: horizon,
///   [], [*Area*], [*Parameters*],
///   image("cylinder.svg"),
///   $ pi h (D^2 - d^2) / 4 $,
///   [
///     $h$: height \
///     $D$: outer radius \
///     $d$: inner radius
///   ],
///   image("tetrahedron.svg"),
///   $ sqrt(2) / 12 a^3 $,
///   [$a$: edge length]
/// )
/// ```
///
/// ## Parameters
/// - cells: `Content` (positional, variadic)
///   The contents of the table cells.
///
/// - rows: `TrackSizings` (named)
///   Defines the row sizes.
///   See the [grid documentation]($func/grid) for more information on track
///   sizing.
///
/// - columns: `TrackSizings` (named)
///   Defines the column sizes.
///   See the [grid documentation]($func/grid) for more information on track
///   sizing.
///
/// - gutter: `TrackSizings` (named)
///   Defines the gaps between rows & columns.
///   See the [grid documentation]($func/grid) for more information on gutters.
///
/// - column-gutter: `TrackSizings` (named)
///   Defines the gaps between columns. Takes precedence over `gutter`.
///   See the [grid documentation]($func/grid) for more information on gutters.
///
/// - row-gutter: `TrackSizings` (named)
///   Defines the gaps between rows. Takes precedence over `gutter`.
///   See the [grid documentation]($func/grid) for more information on gutters.
///
/// ## Category
/// layout
#[func]
#[capable(Layout)]
#[derive(Debug, Hash)]
pub struct TableNode {
    /// Defines sizing for content rows and columns.
    pub tracks: Axes<Vec<Sizing>>,
    /// Defines sizing of gutter rows and columns between content.
    pub gutter: Axes<Vec<Sizing>>,
    /// The content to be arranged in the table.
    pub cells: Vec<Content>,
}

#[node]
impl TableNode {
    /// How to fill the cells.
    ///
    /// This can be a color or a function that returns a color. The function is
    /// passed the cell's column and row index, starting at zero. This can be
    /// used to implement striped tables.
    ///
    /// ```example
    /// #table(
    ///   fill: (col, _) => if calc.odd(col) { luma(240) } else { white },
    ///   align: (col, row) =>
    ///     if row == 0 { center }
    ///     else if col == 0 { left }
    ///     else { right },
    ///   columns: 4,
    ///   [], [*Q1*], [*Q2*], [*Q3*],
    ///   [Revenue:], [1000 €], [2000 €], [3000 €],
    ///   [Expenses:], [500 €], [1000 €], [1500 €],
    ///   [Profit:], [500 €], [1000 €], [1500 €],
    /// )
    /// ```
    #[property(referenced)]
    pub const FILL: Celled<Option<Paint>> = Celled::Value(None);

    /// How to align the cell's content.
    ///
    /// This can either be a single alignment or a function that returns an
    /// alignment. The function is passed the cell's column and row index,
    /// starting at zero. If set to `{auto}`, the outer alignment is used.
    #[property(referenced)]
    pub const ALIGN: Celled<Smart<Axes<Option<GenAlign>>>> = Celled::Value(Smart::Auto);

    /// How to stroke the cells.
    ///
    /// This can be a color, a stroke width, both, or `{none}` to disable
    /// the stroke.
    #[property(resolve, fold)]
    pub const STROKE: Option<PartialStroke> = Some(PartialStroke::default());

    /// How much to pad the cells's content.
    ///
    /// The default value is `{5pt}`.
    pub const INSET: Rel<Length> = Abs::pt(5.0).into();

    fn construct(_: &Vm, args: &mut Args) -> SourceResult<Content> {
        let TrackSizings(columns) = args.named("columns")?.unwrap_or_default();
        let TrackSizings(rows) = args.named("rows")?.unwrap_or_default();
        let TrackSizings(base_gutter) = args.named("gutter")?.unwrap_or_default();
        let column_gutter = args.named("column-gutter")?.map(|TrackSizings(v)| v);
        let row_gutter = args.named("row-gutter")?.map(|TrackSizings(v)| v);
        Ok(Self {
            tracks: Axes::new(columns, rows),
            gutter: Axes::new(
                column_gutter.unwrap_or_else(|| base_gutter.clone()),
                row_gutter.unwrap_or(base_gutter),
            ),
            cells: args.all()?,
        }
        .pack())
    }

    fn field(&self, name: &str) -> Option<Value> {
        match name {
            "columns" => Some(Sizing::encode_slice(&self.tracks.x)),
            "rows" => Some(Sizing::encode_slice(&self.tracks.y)),
            "column-gutter" => Some(Sizing::encode_slice(&self.gutter.x)),
            "row-gutter" => Some(Sizing::encode_slice(&self.gutter.y)),
            "cells" => Some(Value::Array(
                self.cells.iter().cloned().map(Value::Content).collect(),
            )),
            _ => None,
        }
    }
}

impl Layout for TableNode {
    fn layout(
        &self,
        vt: &mut Vt,
        styles: StyleChain,
        regions: Regions,
    ) -> SourceResult<Fragment> {
        let fill = styles.get(Self::FILL);
        let stroke = styles.get(Self::STROKE).map(PartialStroke::unwrap_or_default);
        let inset = styles.get(Self::INSET);
        let align = styles.get(Self::ALIGN);

        let cols = self.tracks.x.len().max(1);
        let cells = self
            .cells
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, child)| {
                let mut child = child.padded(Sides::splat(inset));

                let x = i % cols;
                let y = i / cols;
                if let Smart::Custom(alignment) = align.resolve(vt, x, y)? {
                    child = child.styled(AlignNode::ALIGNS, alignment)
                }

                if let Some(stroke) = stroke {
                    child = child.stroked(stroke);
                }

                if let Some(fill) = fill.resolve(vt, x, y)? {
                    child = child.filled(fill);
                }

                Ok(child)
            })
            .collect::<SourceResult<_>>()?;

        GridNode {
            tracks: self.tracks.clone(),
            gutter: self.gutter.clone(),
            cells,
        }
        .layout(vt, styles, regions)
    }
}

/// A value that can be configured per cell.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Celled<T> {
    /// A bare value, the same for all cells.
    Value(T),
    /// A closure mapping from cell coordinates to a value.
    Func(Func),
}

impl<T: Cast + Clone> Celled<T> {
    /// Resolve the value based on the cell position.
    pub fn resolve(&self, vt: &Vt, x: usize, y: usize) -> SourceResult<T> {
        Ok(match self {
            Self::Value(value) => value.clone(),
            Self::Func(func) => {
                let args =
                    Args::new(func.span(), [Value::Int(x as i64), Value::Int(y as i64)]);
                func.call_detached(vt.world(), args)?.cast().at(func.span())?
            }
        })
    }
}

impl<T: Cast> Cast for Celled<T> {
    fn is(value: &Value) -> bool {
        matches!(value, Value::Func(_)) || T::is(value)
    }

    fn cast(value: Value) -> StrResult<Self> {
        match value {
            Value::Func(v) => Ok(Self::Func(v)),
            v if T::is(&v) => Ok(Self::Value(T::cast(v)?)),
            v => <Self as Cast>::error(v),
        }
    }

    fn describe() -> CastInfo {
        T::describe() + CastInfo::Type("function")
    }
}