// Pip, TuxBridge's penguin: original artwork drawn here (not Tux), so it belongs to the project and
// can be used anywhere the app goes. A round head with a feather tuft, or a graduation cap in Kids
// mode, and a few moods.

export type Mood = "idle" | "happy" | "think" | "wow";

export default function Penguin({ mood = "idle", kids = false, size = 56, className = "" }: { mood?: Mood; kids?: boolean; size?: number; className?: string }) {
  const look = mood === "think" ? -2.5 : 0;
  return (
    <svg viewBox="0 0 100 100" width={size} height={size} className={className} aria-hidden="true">
      {/* tuft, or a cap */}
      {kids ? (
        <g>
          <path d="M50 4 L88 18 L50 32 L12 18 Z" fill="#2b2f3a" />
          <path d="M30 24 L30 34 Q50 42 70 34 L70 24 L50 32 Z" fill="#1d2029" />
          <path d="M50 18 L80 26 L80 40" fill="none" stroke="#ffc83d" strokeWidth="2.5" strokeLinecap="round" />
          <circle cx="80" cy="42" r="3.5" fill="#ffc83d" />
        </g>
      ) : (
        <path d="M44 18 Q42 6 50 4 Q47 10 52 14 Q55 6 62 8 Q56 12 58 20 Z" fill="#2c3a57" />
      )}
      {/* head */}
      <circle cx="50" cy="56" r="40" fill="#2c3a57" />
      {/* face */}
      <path d="M50 36 C41 24 17 30 18 56 C19 79 37 90 50 90 C63 90 81 79 82 56 C83 30 59 24 50 36 Z" fill="#f7f9fd" />
      {/* cheeks */}
      <ellipse cx="27" cy="68" rx="6" ry="3.5" fill="#ff9db0" opacity={kids || mood === "happy" ? 0.75 : 0.45} />
      <ellipse cx="73" cy="68" rx="6" ry="3.5" fill="#ff9db0" opacity={kids || mood === "happy" ? 0.75 : 0.45} />
      {/* eyes */}
      {mood === "happy" ? (
        <g fill="none" stroke="#1b2233" strokeWidth="4" strokeLinecap="round">
          <path d="M30 56 Q37 48 44 56" />
          <path d="M56 56 Q63 48 70 56" />
        </g>
      ) : (
        <g className="pip-eyes">
          <circle cx="37" cy="55" r={mood === "wow" ? 9 : 7.5} fill="#1b2233" />
          <circle cx="63" cy="55" r={mood === "wow" ? 9 : 7.5} fill="#1b2233" />
          <circle cx={39.5 + look} cy={51.5 + look} r="2.6" fill="#fff" />
          <circle cx={65.5 + look} cy={51.5 + look} r="2.6" fill="#fff" />
        </g>
      )}
      {/* beak */}
      {mood === "wow" ? (
        <ellipse cx="50" cy="70" rx="5" ry="6" fill="#ff9f1c" />
      ) : (
        <path d="M42 66 Q50 61 58 66 Q51 76 50 76 Q49 76 42 66 Z" fill="#ff9f1c" />
      )}
    </svg>
  );
}
