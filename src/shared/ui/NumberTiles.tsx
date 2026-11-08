interface Tile {
  value: string
  label: string
}

/** Number tiles with a note beside them (startup and drivers pages). */
export function NumberTiles({ tiles, note }: { tiles: Tile[]; note: string }) {
  return (
    <div className="number-tiles">
      {tiles.map((t) => (
        <div key={t.label} className="number-tile">
          <strong>{t.value}</strong>
          <span>{t.label}</span>
        </div>
      ))}
      <div className="note-box">{note}</div>
    </div>
  )
}
