from PIL import Image, ImageDraw

def make_car():
    img = Image.new('RGBA', (32, 64), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    draw.rounded_rectangle([(4, 4), (28, 60)], radius=4, fill=(220, 80, 80), outline=(50, 50, 50), width=2)
    draw.rectangle([(8, 16), (24, 24)], fill=(100, 200, 255))
    draw.rectangle([(8, 48), (24, 52)], fill=(100, 200, 255))
    draw.ellipse([(6, 2), (12, 6)], fill=(255, 255, 100))
    draw.ellipse([(20, 2), (26, 6)], fill=(255, 255, 100))
    img.save('assets/car.png')

def make_road():
    # Simple asphalt tile
    img = Image.new('RGBA', (64, 64), (72, 74, 78, 255))
    draw = ImageDraw.Draw(img)
    # add some noise
    import random
    for _ in range(100):
        x = random.randint(0, 63)
        y = random.randint(0, 63)
        draw.point((x, y), fill=(80, 82, 86, 255))
    img.save('assets/road.png')

make_car()
make_road()
