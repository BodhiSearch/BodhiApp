/* ═══════════════════════════════════════════════════════════════
   Bodhi Models — shared primitives
   models/models-base.jsx   (load 1st of the models page modules,
                             after bodhi-models-data.js + the shell)

   Tiny helpers shared by every models module: the icon alias and the
   capability/type Tag pill. Published to window for the other
   models-*.jsx files.
═══════════════════════════════════════════════════════════════ */
const { TAG_MAP } = window.MODELS_DATA;

const Ic = ShellIcon;
const tagClass = (t) => 'tag ' + (TAG_MAP[t] || 'tag-muted');
const Tag = ({ t, big }) => <span className={tagClass(t)} style={big ? { fontSize: 12, padding: '3px 9px' } : null}>{t}</span>;

Object.assign(window, { Ic, tagClass, Tag });
