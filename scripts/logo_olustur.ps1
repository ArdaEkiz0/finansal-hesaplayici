# Finansal Hesaplayici - vector logo source and multi-resolution ICO generator.
# ASCII-only source: Windows PowerShell 5.1 reads UTF-8-no-BOM as ANSI.
Add-Type -AssemblyName System.Drawing

$kok = Split-Path -Parent $PSScriptRoot
$build = Join-Path $kok 'build'
New-Item -ItemType Directory -Force $build | Out-Null

$lira = [string][char]0x20BA

function New-IconBitmap([int]$s, [bool]$basit) {
    $b = New-Object System.Drawing.Bitmap($s, $s)
    $g = [System.Drawing.Graphics]::FromImage($b)
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit

    $k = [double]$s / 512.0
    $kenarB = [int][Math]::Round(12 * $k)
    $r = New-Object System.Drawing.Rectangle($kenarB, $kenarB, ($s - 2 * $kenarB), ($s - 2 * $kenarB))
    $rad = [int][Math]::Round(104 * $k)
    if ($rad -lt 2) { $rad = 2 }
    $p = New-Object System.Drawing.Drawing2D.GraphicsPath
    $p.AddArc($r.Left, $r.Top, $rad, $rad, 180, 90)
    $p.AddArc($r.Right - $rad, $r.Top, $rad, $rad, 270, 90)
    $p.AddArc($r.Right - $rad, $r.Bottom - $rad, $rad, $rad, 0, 90)
    $p.AddArc($r.Left, $r.Bottom - $rad, $rad, $rad, 90, 90)
    $p.CloseFigure()

    $grad = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
        $r,
        [System.Drawing.Color]::FromArgb(255, 24, 40, 76),
        [System.Drawing.Color]::FromArgb(255, 7, 13, 27),
        82)
    $g.FillPath($grad, $p)
    $cizgi = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 47, 123, 255), [Math]::Max(1, [int][Math]::Round(5 * $k)))
    $g.DrawPath($cizgi, $p)

    $bicim = New-Object System.Drawing.StringFormat
    $bicim.Alignment = [System.Drawing.StringAlignment]::Center
    $bicim.LineAlignment = [System.Drawing.StringAlignment]::Center
    $beyaz = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::White)

    if ($basit) {
        # Kucuk boyut: sadece buyuk ₺, izgara yok (16px'de okunur kalsin).
        $yazi = New-Object System.Drawing.Font('Segoe UI', [Math]::Max(6, [int][Math]::Round($s * 0.80)), [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
        $g.DrawString($lira, $yazi, $beyaz, (New-Object System.Drawing.RectangleF(0, 0, $s, $s)), $bicim)
    }
    else {
        $parlak = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(210, 34, 211, 238), [Math]::Max(1, [int][Math]::Round(3 * $k)))
        $g.DrawArc($parlak, $r.Left + 6 * $k, $r.Top + 6 * $k, ($N - 24) * $k, ($N - 24) * $k, 200, 82)
        $izgara = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(170, 34, 211, 238), [Math]::Max(1, [int][Math]::Round(5 * $k)))
        foreach ($y in @(348, 384, 420)) {
            $x2 = 356
            if ($y -eq 384) { $x2 = 300 }
            $g.DrawLine($izgara, [single](156 * $k), [single]($y * $k), [single]($x2 * $k), [single]($y * $k))
        }
        $yazi = New-Object System.Drawing.Font('Segoe UI', [Math]::Max(8, [int][Math]::Round(244 * $k)), [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
        $kare = (New-Object System.Drawing.RectangleF(0, [single](22 * $k), [single]$s, [single](340 * $k)))
        $g.DrawString($lira, $yazi, $beyaz, $kare, $bicim)
    }
    $g.Dispose()
    return $b
}

$N = 512
$bmp = New-IconBitmap $N $false
$png = Join-Path $build 'logo.png'
$bmp.Save($png, [System.Drawing.Imaging.ImageFormat]::Png)

# ICO: her boyut PNG olarak gomulur (Vista+), 256 dahil.
$boyutlar = @(16, 24, 32, 48, 64, 128, 256)
$parcalar = [System.Collections.Generic.List[byte]]::new()
$dir = [byte[]](0, 0, 1, 0, $boyutlar.Count, 0)
$ofset = 6 + (16 * $boyutlar.Count)
foreach ($s in $boyutlar) {
    $kucuk = New-IconBitmap $s ($s -le 48)
    $ms = New-Object System.IO.MemoryStream
    $kucuk.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $veri = $ms.ToArray()
    $ms.Dispose(); $kucuk.Dispose()
    $w = if ($s -eq 256) { 0 } else { $s }
    $dir += [byte[]]($w, $w, 0, 0, 1, 0, 32, 0)
    $dir += [BitConverter]::GetBytes([int]$veri.Length)
    $dir += [BitConverter]::GetBytes([int]$ofset)
    $ofset += $veri.Length
    $parcalar.AddRange($veri)
}
$ico = Join-Path $build 'icon.ico'
$fs = [System.IO.File]::Create($ico)
$fs.Write($dir, 0, $dir.Length)
$fs.Write($parcalar.ToArray(), 0, $parcalar.Count)
$fs.Dispose()
$bmp.Dispose()
"logo: $png"
"ico : $ico ($((Get-Item $ico).Length) bytes, $($boyutlar.Count) sizes)"
