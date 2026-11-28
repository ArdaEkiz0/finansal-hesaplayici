# Uc logo taslagi uretir (ASCII-only, System.Drawing).
Add-Type -AssemblyName System.Drawing
$cikti = (Join-Path $env:TEMP "logo-secenekler")
New-Item -ItemType Directory -Force $cikti | Out-Null
$lira = [string][char]0x20BA
$N = 512

function Ciz-Tile($g, [System.Drawing.Rectangle]$r, [System.Drawing.Color]$ust, [System.Drawing.Color]$alt, [System.Drawing.Color]$kenarRengi) {
    $rad = 104
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $p.AddArc($r.Left, $r.Top, $rad, $rad, 180, 90)
    $p.AddArc($r.Right - $rad, $r.Top, $rad, $rad, 270, 90)
    $p.AddArc($r.Right - $rad, $r.Bottom - $rad, $rad, $rad, 0, 90)
    $p.AddArc($r.Left, $r.Bottom - $rad, $rad, $rad, 90, 90)
    $p.CloseFigure()
    $grad = New-Object System.Drawing.Drawing2D.LinearGradientBrush($r, $ust, $alt, 82)
    $g.FillPath($grad, $p)
    $g.DrawPath((New-Object System.Drawing.Pen($kenarRengi, 5)), $p)
}

function Yazi-Ciz($g, [string]$metin, [int]$boyut, [int]$y) {
    $f = New-Object System.Drawing.Font('Segoe UI', $boyut, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $kare = (New-Object System.Drawing.RectangleF(0, $y, 512, $(512 - $y)))
    $g.DrawString($metin, $f, [System.Drawing.SolidBrush]::new([System.Drawing.Color]::White), $kare, $sf)
}

# 1 - Lira + defter izgarasi (mevcut)
$b = New-Object System.Drawing.Bitmap($N, $N)
$g = [System.Drawing.Graphics]::FromImage($b)
$g.Clear([System.Drawing.Color]::Transparent)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
Ciz-Tile $g (New-Object System.Drawing.Rectangle(12, 12, 488, 488)) ([System.Drawing.Color]::FromArgb(255, 24, 40, 76)) ([System.Drawing.Color]::FromArgb(255, 7, 13, 27)) ([System.Drawing.Color]::FromArgb(255, 47, 123, 255))
$iz = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(170, 34, 211, 238), 5)
$g.DrawLine($iz, 156, 348, 356, 348); $g.DrawLine($iz, 156, 384, 300, 384); $g.DrawLine($iz, 156, 420, 332, 420)
Yazi-Ciz $g $lira 244 22
$g.Dispose(); $b.Save("$cikti\logo-1.png", [System.Drawing.Imaging.ImageFormat]::Png); $b.Dispose()

# 2 - Kalkulator: buyuk ₺ + tus noktalari
$b = New-Object System.Drawing.Bitmap($N, $N)
$g = [System.Drawing.Graphics]::FromImage($b)
$g.Clear([System.Drawing.Color]::Transparent)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
Ciz-Tile $g (New-Object System.Drawing.Rectangle(12, 12, 488, 488)) ([System.Drawing.Color]::FromArgb(255, 30, 86, 190)) ([System.Drawing.Color]::FromArgb(255, 12, 30, 78)) ([System.Drawing.Color]::FromArgb(180, 34, 211, 238))
Yazi-Ciz $g $lira 250 0
$fw = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(230, 125, 211, 252))
foreach ($x in @(170, 256, 342)) { foreach ($y in @(372, 424)) { $g.FillEllipse($fw, ($x - 11), ($y - 11), 22, 22) } }
$g.Dispose(); $b.Save("$cikti\logo-2.png", [System.Drawing.Imaging.ImageFormat]::Png); $b.Dispose()

# 3 - Monogram: ₺ beyaz daire icinde, cyan diagonals
$b = New-Object System.Drawing.Bitmap($N, $N)
$g = [System.Drawing.Graphics]::FromImage($b)
$g.Clear([System.Drawing.Color]::Transparent)
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
$g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
Ciz-Tile $g (New-Object System.Drawing.Rectangle(12, 12, 488, 488)) ([System.Drawing.Color]::FromArgb(255, 16, 27, 54)) ([System.Drawing.Color]::FromArgb(255, 6, 12, 26)) ([System.Drawing.Color]::FromArgb(255, 47, 123, 255))
$cy = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 34, 211, 238))
$g.FillRectangle($cy, 40, 400, 430, 10)
$g.FillRectangle($cy, 40, 436, 300, 10)
$f = New-Object System.Drawing.Font('Segoe UI', 220, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = [System.Drawing.StringAlignment]::Center
$sf.LineAlignment = [System.Drawing.StringAlignment]::Center
$g.DrawString($lira, $f, [System.Drawing.SolidBrush]::new([System.Drawing.Color]::White), (New-Object System.Drawing.RectangleF(0, 0, 512, 380)), $sf)
$g.Dispose(); $b.Save("$cikti\logo-3.png", [System.Drawing.Imaging.ImageFormat]::Png); $b.Dispose()

"1: $cikti\logo-1.png"
"2: $cikti\logo-2.png"
"3: $cikti\logo-3.png"
