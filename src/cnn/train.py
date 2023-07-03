import glob
import os
import random
import re
import warnings
from math import ceil

import keras
import numpy as np
from keras import backend as K
from keras import layers, models
from keras.callbacks import EarlyStopping, ModelCheckpoint, ReduceLROnPlateau
from numpy.random import seed
from skimage import io, transform
from skimage.util.shape import view_as_blocks
from sklearn.model_selection import KFold
from tensorflow.random import set_seed
from tqdm import tqdm_notebook as tqdm

warnings.filterwarnings("ignore")


SQUARE_SIZE = 40  # must be less than 400/8==50
train_size = 3000
test_size = 500
BATCH_SIZE = 128
SEED = 2019
Epoch = 15
k_folds = 5
PATIENCE = 5

random.seed(SEED)


def get_image_filenames(image_path, image_type):
    if os.path.exists(image_path):
        return glob.glob(os.path.join(image_path, "*." + image_type))
    return


seed(SEED)

set_seed(SEED)
SQUARE_SIZE = 40

DATA_PATH = "/home/leghart/projects/chessify_utils/archive/dataset"
TRAIN_IMAGE_PATH = os.path.join(DATA_PATH, "train")
TEST_IMAGE_PATH = os.path.join(DATA_PATH, "test")


train = get_image_filenames(TRAIN_IMAGE_PATH, "jpeg")
test = get_image_filenames(TEST_IMAGE_PATH, "jpeg")

random.shuffle(train)
random.shuffle(test)

train = train[:train_size]
test = test[:test_size]

piece_symbols = "prbnkqPRBNKQ"


def _fen_from_filename(filename):
    base = os.path.basename(filename)
    return os.path.splitext(base)[0]


def _onehot_from_fen(fen):
    eye = np.eye(13)
    output = np.empty((0, 13))
    fen = re.sub("[-]", "", fen)

    for char in fen:
        if char in "12345678":
            output = np.append(output, np.tile(eye[12], (int(char), 1)), axis=0)
        else:
            idx = piece_symbols.index(char)
            output = np.append(output, eye[idx].reshape((1, 13)), axis=0)

    return output


def fen_from_onehot(one_hot):
    output = ""
    for j in range(8):
        for i in range(8):
            if one_hot[j][i] == 12:
                output += " "
            else:
                output += piece_symbols[one_hot[j][i]]
        if j != 7:
            output += "-"

    for i in range(8, 0, -1):
        output = output.replace(" " * i, str(i))

    return output


def process_image(img):
    downsample_size = SQUARE_SIZE * 8
    square_size = SQUARE_SIZE

    img_read = io.imread(img)

    img_read = transform.resize(img_read, (downsample_size, downsample_size), mode="constant")

    tiles = view_as_blocks(img_read, block_shape=(square_size, square_size, 3))
    tiles = tiles.squeeze(axis=2)
    return tiles.reshape(64, square_size, square_size, 3)


def process_image_bytes(img_read):
    downsample_size = SQUARE_SIZE * 8
    square_size = SQUARE_SIZE

    img_read = transform.resize(img_read, (downsample_size, downsample_size), mode="constant")

    tiles = view_as_blocks(img_read, block_shape=(square_size, square_size, 3))
    tiles = tiles.squeeze(axis=2)
    return tiles.reshape(64, square_size, square_size, 3)


def _train_gen(features, batch_size):
    i = 0
    while True:
        batch_x = []
        batch_y = []
        for b in range(batch_size):
            if i == len(features):
                i = 0
                random.shuffle(features)
            img = str(features[i])
            y = _onehot_from_fen(_fen_from_filename(img))
            x = process_image(img)
            for x_part in x:
                batch_x.append(x_part)
            for y_part in y:
                batch_y.append(y_part)
            i += 1
        yield (np.array(batch_x), np.array(batch_y))


def _pred_gen(features, batch_size):
    for i, img in enumerate(features):
        yield process_image(img)


def _get_callbacks(model_name, patient):
    ES = EarlyStopping(monitor="val_loss", patience=patient, mode="min", verbose=1)
    RR = ReduceLROnPlateau(
        monitor="val_loss", factor=0.5, patience=patient / 2, min_lr=0.000001, verbose=1, mode="min"
    )
    MC = ModelCheckpoint(filepath=model_name, monitor="val_loss", verbose=1, save_best_only=True, mode="min")

    return [ES, RR, MC]


def weighted_categorical_crossentropy(weights):
    """
    A weighted version of keras.objectives.categorical_crossentropy

    Variables:
        weights: numpy array of shape (C,) where C is the number of classes

    Usage:
        weights = np.array([0.5,2,10]) # Class one at 0.5, class 2 twice the normal weights, class 3 10x.
        loss = weighted_categorical_crossentropy(weights)
        model.compile(loss=loss,optimizer='adam')
    """

    weights = K.variable(weights)

    def loss(y_true, y_pred):
        # scale predictions so that the class probas of each sample sum to 1
        y_pred /= K.sum(y_pred, axis=-1, keepdims=True)
        # clip to prevent NaN's and Inf's
        y_pred = K.clip(y_pred, K.epsilon(), 1 - K.epsilon())
        # calc
        loss = y_true * K.log(y_pred) * weights
        loss = -K.sum(loss, -1)
        return loss

    return loss


def _get_model(image_size):
    model = models.Sequential()
    model.add(
        layers.Conv2D(
            32, (3, 3), activation="relu", kernel_initializer="he_normal", input_shape=(image_size, image_size, 3)
        )
    )
    model.add(layers.Dropout(0.2))
    model.add(layers.Conv2D(32, (3, 3), activation="relu", kernel_initializer="he_normal"))
    model.add(layers.Dropout(0.2))
    model.add(layers.MaxPooling2D(pool_size=(2, 2), padding="same"))
    model.add(layers.Conv2D(32, (3, 3), activation="relu", kernel_initializer="he_normal"))
    model.add(layers.Dropout(0.2))
    model.add(layers.Conv2D(32, (3, 3), activation="relu", kernel_initializer="he_normal"))
    model.add(layers.Dropout(0.2))
    model.add(layers.Flatten())
    model.add(layers.Dense(128, activation="relu", kernel_initializer="he_normal"))
    model.add(layers.Dropout(0.2))
    model.add(layers.Dense(13, activation="softmax", kernel_initializer="lecun_normal"))

    #    model.summary()

    weights = np.array(
        [
            1 / (0.30 * 4),
            1 / (0.20 * 4),
            1 / (0.20 * 4),
            1 / (0.20 * 4),
            1 / 1,
            1 / (0.10 * 4),
            1 / (0.30 * 4),
            1 / (0.20 * 4),
            1 / (0.20 * 4),
            1 / (0.20 * 4),
            1 / 1,
            1 / (0.10 * 4),
            1 / (64 - 10),
        ]
    )
    model.compile(
        loss=weighted_categorical_crossentropy(weights), optimizer="nadam", metrics=["acc"]
    )  # weight the inverse of expected pieces

    return model


def start_train() -> None:
    kf = KFold(n_splits=k_folds, random_state=SEED, shuffle=True)

    j = 1
    model_names = []
    for train_fold, valid_fold in kf.split(train):
        print("=========================================")
        print("====== K Fold Validation step => %d/%d =======" % (j, k_folds))
        print("=========================================")

        model_name = "./" + str(j) + ".hdf5"
        model_names.append(model_name)
        model = _get_model(SQUARE_SIZE)

        model.fit_generator(
            _train_gen([train[i] for i in tqdm(train_fold)], batch_size=BATCH_SIZE),
            steps_per_epoch=ceil(train_size * (1 - 1 / k_folds) / BATCH_SIZE),
            epochs=Epoch,
            validation_data=_train_gen([train[i] for i in tqdm(valid_fold)], batch_size=BATCH_SIZE),
            validation_steps=ceil(train_size / k_folds / BATCH_SIZE),
            verbose=1,
            shuffle=False,
            callbacks=_get_callbacks(model_name, PATIENCE),
        )
        j += 1  # single batch is actually 64*batch_size, since there are 64 pieces on the board

    for name in tqdm(model_names):
        res = (
            (
                keras.models.load_model(
                    name,
                    custom_objects={
                        "loss": weighted_categorical_crossentropy(
                            np.array(
                                [
                                    1 / (0.30 * 4),
                                    1 / (0.20 * 4),
                                    1 / (0.20 * 4),
                                    1 / (0.20 * 4),
                                    1 / 1,
                                    1 / (0.10 * 4),
                                    1 / (0.30 * 4),
                                    1 / (0.20 * 4),
                                    1 / (0.20 * 4),
                                    1 / (0.20 * 4),
                                    1 / 1,
                                    1 / (0.10 * 4),
                                    1 / (64 - 10),
                                ]
                            )
                        )
                    },
                )
            )
            .predict_generator(_pred_gen(test, 64), steps=test_size)
            .argmax(axis=1)
            .reshape(-1, 8, 8)
        )
        pred_fens = np.array([fen_from_onehot(one_hot) for one_hot in res])
        test_fens = np.array([_fen_from_filename(fn) for fn in test])

        final_accuracy = (pred_fens == test_fens).astype(float).mean()

        print("Model Name: ", name, "Final Accuracy: {:1.5f}%".format(final_accuracy))

        res_stacked = model.predict_generator(_pred_gen(test, 64), steps=test_size).argmax(axis=1).reshape(-1, 8, 8)

        pred_fens = np.array([fen_from_onehot(one_hot) for one_hot in res_stacked])
        test_fens = np.array([_fen_from_filename(fn) for fn in test])

        final_accuracy = (pred_fens == test_fens).astype(float).mean()

        print("Final Accuracy: {:1.5f}%".format(final_accuracy))
