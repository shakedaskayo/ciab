interface Props {
  size?: number;
  className?: string;
}

export default function CiabLogo({ size = 32, className }: Props) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      {/* Glass box — outer border */}
      <rect
        x="5"
        y="16"
        width="54"
        height="43"
        rx="4"
        stroke="#C4693D"
        strokeWidth="2"
        fill="none"
        opacity="0.4"
      />
      {/* Glass box — fill */}
      <rect
        x="5"
        y="16"
        width="54"
        height="43"
        rx="4"
        fill="#C4693D"
        opacity="0.05"
      />
      {/* Glass shine — top edge highlight */}
      <line x1="9" y1="16" x2="22" y2="16" stroke="#C4693D" strokeWidth="2.5" opacity="0.6" strokeLinecap="round" />
      {/* Glass shine — side glint */}
      <line x1="5" y1="20" x2="5" y2="28" stroke="#C4693D" strokeWidth="2" opacity="0.2" strokeLinecap="round" />

      {/* Creature body — rounded square */}
      <rect x="18" y="24" width="28" height="26" rx="5" fill="#C4693D" />
      {/* Body highlight */}
      <rect x="18" y="24" width="28" height="13" rx="5" fill="#D4845E" opacity="0.3" />

      {/* Eyes */}
      <rect x="24" y="31" width="5" height="5" rx="1.5" fill="#0F1117" />
      <rect x="35" y="31" width="5" height="5" rx="1.5" fill="#0F1117" />
      {/* Eye shine */}
      <rect x="25.5" y="32" width="2" height="2" rx="0.75" fill="#fff" opacity="0.8" />
      <rect x="36.5" y="32" width="2" height="2" rx="0.75" fill="#fff" opacity="0.8" />

      {/* Mouth — subtle smile */}
      <path d="M27 42 C30 44.5 34 44.5 37 42" stroke="#0F1117" strokeWidth="2" strokeLinecap="round" fill="none" opacity="0.5" />

      {/* Feet */}
      <rect x="20" y="50" width="9" height="6" rx="3" fill="#A85530" />
      <rect x="35" y="50" width="9" height="6" rx="3" fill="#A85530" />

      {/* Antennae */}
      <line x1="27" y1="24" x2="27" y2="12" stroke="#C4693D" strokeWidth="2.5" strokeLinecap="round" />
      <line x1="37" y1="24" x2="37" y2="12" stroke="#C4693D" strokeWidth="2.5" strokeLinecap="round" />
      {/* Antenna tips — glowing orbs */}
      <circle cx="27" cy="10" r="3.5" fill="#D4845E" />
      <circle cx="37" cy="10" r="3.5" fill="#D4845E" />
      <circle cx="26" cy="9" r="1.2" fill="#fff" opacity="0.4" />
      <circle cx="36" cy="9" r="1.2" fill="#fff" opacity="0.4" />
    </svg>
  );
}
